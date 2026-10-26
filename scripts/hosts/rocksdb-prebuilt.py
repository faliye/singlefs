#!/usr/bin/env python3
# admission: always 判的是此刻这台主机的工具链、预编库与环境，上一次的结论不替这一次作保
# run-condition: command python3 rustc cargo c++ cc ar
"""librocksdb-sys 的 C++（librocksdb.a 与 libsnappy.a）每台主机只编一次，各编译目录经环境变量链这一份，不在每个编译目录里现编。

用法（都在仓根跑；--root 不给是这份脚本所在的仓）：
  rocksdb-prebuilt.py key [--root R]           打这台主机此刻的键（各组成）与预编库目录，不编
  rocksdb-prebuilt.py env [--root R]           预编库在、对得上时打四行 export（eval 用）；不在退 1、标准输出为空
  rocksdb-prebuilt.py build [--root R]         预编库不在就编一次（持锁：几个进程同时来只编一次，后来的等锁之后看到编好了直接退 0）；起真 cargo，是重活
  rocksdb-prebuilt.py install [--root R]       build，再写这台主机的三处环境来源（见下「入口怎么拿到取值」）、改用户级 systemd 管理器此刻的环境，最后跑 check
  rocksdb-prebuilt.py check [--root R]         判这台主机：两个工作区的依赖对得上、预编库在且对得上、三处环境来源与此刻进程环境的取值都与预编库一致、cargo 配置里没写这几个变量
  rocksdb-prebuilt.py entry [--root R] [--cargo C]
                                               起 cargo 的入口（门禁阶段、崩溃与变异驱动、merge-and-verify.sh、admission.py 交的命令、第二台那一侧）开头调的轻判：
                                               只判这台装了（预编库在、对得上，不算两份库的 sha256）、此刻进程环境的取值与它一致、没设 ROCKSDB_COMPILE / SNAPPY_COMPILE；
                                               不判三处环境来源（那是 check 的事）。判红退 1、逐项打 ✗ 与 → 出路，入口照它退出、一个 cargo 都不起。见下「entry」
  rocksdb-prebuilt.py audit-target <编译目录>… [--root R]
                                               判编译目录：librocksdb-sys 构建脚本那一次拿到的取值是不是这台的预编库、out/ 里有没有现编过的 C++、同一把键下见过几种取值
  rocksdb-prebuilt.py --selftest               自证（假 rustc、cargo、c++、cc、ar、systemctl 放在 PATH 最前面，临时目录当 HOME 与库目录，不编译）

预编库放哪：${SINGLEFS_ROCKSDB_PREBUILT_STORE:-~/.local/share/singlefs/rocksdb-prebuilt}/librocksdb-sys-<版本>/<键的前 16 位>/lib/{librocksdb.a,libsnappy.a}，
  同目录 manifest.json 记键的各组成、两份库的 sha256 与大小、编的用时与日期，complete 最后写（有它才算编好）。不放开机清空的目录。
  SINGLEFS_ROCKSDB_PREBUILT_STORE 只给自证换临时目录用。
键（manifest.json 的 fields，sha256 取前 16 位）：
  两个工作区（crates/singlefs-checker-tier/Cargo.toml 与 scripts/experiment-apparatus/Cargo.toml）的 rocksdb 依赖写法（版本、默认特性、特性），两份不同拒；
  两份 Cargo.lock 里 rocksdb、librocksdb-sys、zstd-sys、lz4-sys、libz-sys、bzip2-sys、cc 的版本与 checksum，两份不同拒；
  rustc -vV 原样；c++、cc、ar 解析到的真实路径与 --version 第一行；交给 C++ 的指令集（CPU_FEATURES 这台 /proc/cpuinfo 里缺一样就拒，OPTIONAL_CPU_FEATURES 这台有的才加）；
  编法（opt-level 3、不带调试信息）。
怎么编：在库目录底下的临时目录里建一个只依赖 rocksdb（照仓里的写法）的试编工程，拷仓根 Cargo.lock 进去，cargo build --release --offline，
  环境里拿掉 ROCKSDB_*、SNAPPY_*、RUSTFLAGS 这一类、CC / CXX / CFLAGS / CXXFLAGS 这一类，设 ROCKSDB_COMPILE=1、SNAPPY_COMPILE=1、
  RUSTFLAGS=-C target-feature=<键里的指令集>（librocksdb-sys 照它给 C++ 加 -msse4.1 -msse4.2 -mpclmul，这台有的再加 -mavx2 -mbmi -mlzcnt）、BINDGEN_EXTRA_CLANG_ARGS 取仓里 .cargo/config.toml 的那一份；
  编完核试编工程的 Cargo.lock 里键里那几个包与仓里逐个相同、构建脚本输出里没有「compiling without SSE4.2」，再拷两份 .a、写 manifest.json 与 complete，整个目录改名换上。
  线程数照 cargo 自己读的 CARGO_BUILD_JOBS（经 bash scripts/hosts/capped.sh <N> 起）。
入口怎么拿到取值（四个变量 ROCKSDB_LIB_DIR、ROCKSDB_STATIC、SNAPPY_LIB_DIR、SNAPPY_STATIC，只走进程环境，不写进任何一层 cargo 配置：
  cargo 配置的 [env] 进 crates 变异的行键与广谱变异的全局键，scripts/mutation/lib_mutation_units.py 的 toolchain_lines、scripts/mutation/mutation-prune.py 的 build_environment_lines）：
  ~/.config/singlefs/rocksdb-prebuilt.env       四行 export，唯一的取值来源（install 写）
  ~/.bashrc 最前面一段（BASHRC_BEGIN 到 BASHRC_END） source 上面那一份：交互 shell、从它起的会话与 ssh 起的非交互命令都读（放在交互判断之前）
  ~/.config/environment.d/60-singlefs-rocksdb-prebuilt.conf  用户级 systemd 管理器起的单元（scripts/hosts/detached-run.sh 起的长活）读；
    install 另跑 systemctl --user set-environment 让此刻在跑的管理器当场换上
  ROCKSDB_COMPILE、SNAPPY_COMPILE 一律不设（设成 1 就在编译目录里现编）。
entry（R 是 cargo 要编的那棵树的仓根，默认这份脚本所在的仓；C 是入口要起的 cargo，默认 PATH 上解到的 cargo）：
  不判、退 0 的两种，打一行「· rocksdb-prebuilt entry：… 不判」：
    R 里 crates/singlefs-checker-tier/Cargo.toml 与 scripts/experiment-apparatus/Cargo.toml 都不在、或都没有 rocksdb 依赖（编不到 librocksdb-sys；门禁样本与自证的小仓多是这样）；
    只在自证与样本里认的跳过：C 解到的是一份脚本（#! 开头），头 4096 字节里有一行带 ENTRY_FAKE_CARGO_MARKER（「rocksdb-prebuilt-entry: selftest-fake-cargo」）。
      入口自证与门禁样本里的假 cargo 写这一行注释就不受这道判挡；真的 cargo（rustup 的代理与工具链里的都是 ELF）写不进这一行，带了 C 也照判。
      入口自证里「取值缺了当场退出、一个 cargo 都没起」那一格用一份不带这一行的假 cargo，判照起。
  照判时：先看进程环境（ROCKSDB_COMPILE、SNAPPY_COMPILE 设了、四个取值缺一个，当场红，不往下算键），再算键（只用树里依赖 rocksdb 的那几个工作区的清单与锁：
  只拷了 crates/、Cargo.toml、Cargo.lock 的仓副本照判，各组成取头一个工作区，与两个都在时算出的键相同）、看预编库在不在（store_problem，不算 sha256）、
  比进程环境与预编库的取值。出路：此刻进程没有取值的「开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"」；这台还没装的先 install。
退出码：0 做完 / 全绿；1 判红或没做成（逐项打 ✗ 与 → 出路）；2 用法错；77 audit-target 一份构建脚本记录都没找到（本次未判）。

弄坏开关 ROCKSDB_PREBUILT_BREAK=<项>（逗号分隔，只给 --selftest 证红用）。逐项写弄坏哪一处、打开后自证哪一格红（「✗ 自证 <格名>：」那一段）：
  key-ignores-compiler        键里不放 c++ / cc / ar           「换了 C++ 编译器键就变」「换了编译器之后旧的取值 check 判红（要编新键）」
  key-ignores-lock            键里不放 Cargo.lock 的包         「换了 librocksdb-sys 版本键就变」；键里没有锁，build 也不核试编工程的锁，
                                                               连带「试编工程的依赖版本与仓里对不上不落库」与 build 那几格（对不上的那一份落了库，后面不再起 cargo）
  build-unlocked              build 不持锁                      「两个 build 同时来只编一次」「编好了再 build 不编」（起了两次 cargo）
  unverified-lock-publishes   build 不核试编工程的 Cargo.lock    「试编工程的依赖版本与仓里对不上不落库」，连带「试编失败不落库、不留临时目录」
                                                               「两个 build 同时来只编一次」「编好了再 build 不编」「试编交给 C++ 的指令集是键里的那几样」（对不上的那一份已经落了库）
  missing-store-passes        check 不判预编库在不在            「预编库不在 check 判红」「换了编译器之后旧的取值 check 判红（要编新键）」
  environment-unchecked       check 不比此刻进程环境            「进程环境里的取值与预编库不同 check 判红」「进程环境里设了 ROCKSDB_COMPILE check 判红」
  fingerprint-ignored         audit-target 不读构建脚本的指纹   「编译目录没拿到预编库的取值 audit-target 判红」
  residue-ignored             audit-target 不看 out/ 里现编的残留 「同一编译目录两次取值不同（现编过又换回预编）audit-target 判红」
  ledger-ignored              audit-target 不读不记见过的取值   「同一把键下同一编译目录见过两种取值 audit-target 判红」
  entry-environment-unchecked entry 不看此刻进程环境            「entry：进程环境没有取值判红、不往下算键、出路是开新会话或 eval env」「entry：取值与预编库不同判红」
                                                               「entry：设了 ROCKSDB_COMPILE 判红」「entry：装好、取值对得上判绿」（少判两项）「entry：--cargo 点名的假 cargo 不带标记照判」
                                                               「entry：只拷了 crates/ 的仓副本照判、键与两个工作区都在时相同」（少判两项）
  entry-skips-unmarked-cargo  entry 见 cargo 是脚本就跳过、不看标记 「entry：进程环境没有取值判红、不往下算键、出路是开新会话或 eval env」「entry：取值与预编库不同判红」
                                                               「entry：设了 ROCKSDB_COMPILE 判红」「entry：装好、取值对得上判绿」「entry：--cargo 点名的假 cargo 不带标记照判」
                                                               「entry：只拷了 crates/ 的仓副本照判、键与两个工作区都在时相同」
                                                               （PATH 最前面的假 cargo 与点名的那一份都是不带标记的脚本，跳过了就不判）
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import datetime  # noqa: E402
import fcntl  # noqa: E402
import glob  # noqa: E402
import hashlib  # noqa: E402
import json  # noqa: E402
import os  # noqa: E402
import re  # noqa: E402
import shlex  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import time  # noqa: E402
import tomllib  # noqa: E402
import zoneinfo  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
SCRIPT_RELATIVE = "scripts/hosts/rocksdb-prebuilt.py"
BREAK_VARIABLE = "ROCKSDB_PREBUILT_BREAK"
BREAK_SWITCHES = ("key-ignores-compiler", "key-ignores-lock", "build-unlocked", "unverified-lock-publishes", "missing-store-passes",
                  "environment-unchecked", "fingerprint-ignored", "residue-ignored", "ledger-ignored",
                  "entry-environment-unchecked", "entry-skips-unmarked-cargo")
ENTRY_FAKE_CARGO_MARKER = "rocksdb-prebuilt-entry: selftest-fake-cargo"   # 文件头「entry」：自证与样本的假 cargo 写这一行注释，entry 不判
ENTRY_HOWTO = (f'开新会话（install 之后起的登录 shell 读 ~/.bashrc 最前面那一段），或命令前 eval "$(python3 {SCRIPT_RELATIVE} env)"；'
               f"这台还没装就先 bash scripts/hosts/capped.sh <N> python3 {SCRIPT_RELATIVE} install")
STORE_VARIABLE = "SINGLEFS_ROCKSDB_PREBUILT_STORE"
CPUINFO_VARIABLE = "SINGLEFS_ROCKSDB_PREBUILT_CPUINFO"   # 只给自证换假的 /proc/cpuinfo
MANIFESTS = ("crates/singlefs-checker-tier/Cargo.toml", "scripts/experiment-apparatus/Cargo.toml")
LOCKS = ("Cargo.lock", "scripts/experiment-apparatus/Cargo.lock")
KEY_PACKAGES = ("rocksdb", "librocksdb-sys", "zstd-sys", "lz4-sys", "libz-sys", "bzip2-sys", "cc")
COMPILERS = ("c++", "cc", "ar")
# 交给 C++ 的指令集（rustc 的特性名，/proc/cpuinfo 的位名）：librocksdb-sys 照构建脚本的 CARGO_CFG_TARGET_FEATURE 加 -msse4.1 -msse4.2 -mpclmul -mavx2 -mbmi -mlzcnt。
# CPU_FEATURES 是必有的一档（CRC32C 走 SSE4.2 与 PCLMUL），这台缺就拒；OPTIONAL_CPU_FEATURES 这台有才加（块校验和 XXH3 与布隆过滤器按 __AVX2__ 编），进键。
# 每台各编各的、编出来的二进制不跨机器拷（多机拷的是树，不带编译目录），所以按这台有的取；不取 -march=native（另开 AVX-512、按型号调度，进键的组成说不清）。
CPU_FEATURES = (("sse4.1", "sse4_1"), ("sse4.2", "sse4_2"), ("pclmulqdq", "pclmulqdq"))
OPTIONAL_CPU_FEATURES = (("avx2", "avx2"), ("bmi1", "bmi1"), ("lzcnt", "abm"))
BUILD_PROFILE = {"opt-level": 3, "debug": False}
LINK_VARIABLES = ("ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC")
COMPILE_VARIABLES = ("ROCKSDB_COMPILE", "SNAPPY_COMPILE")
WATCHED_VARIABLES = LINK_VARIABLES + COMPILE_VARIABLES
ARCHIVES = ("librocksdb.a", "libsnappy.a")
BASHRC_BEGIN = "# >>> singlefs rocksdb-prebuilt >>>"
BASHRC_END = "# <<< singlefs rocksdb-prebuilt <<<"
INTERACTIVE_GUARD = re.compile(r'^\s*(case\s+\$-\s+in|\[\s*-z\s+"?\$PS1"?\s*\]|\[\[\s*\$-\s*!=|return\b)')
FINGERPRINT_GLOBS = ("*/.fingerprint/librocksdb-sys-*/run-build-script-build-script-build.json",
                     "*/*/.fingerprint/librocksdb-sys-*/run-build-script-build-script-build.json")
EXIT_NOT_RUN = 77
STRIPPED_PREFIXES = ("ROCKSDB_", "SNAPPY_", "CARGO_PROFILE_", "CARGO_TARGET_", "CC_", "CXX_", "CFLAGS_", "CXXFLAGS_", "AR_",
                     "HOST_", "TARGET_CC", "TARGET_CXX", "TARGET_CFLAGS", "TARGET_CXXFLAGS", "TARGET_AR")
STRIPPED_NAMES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_TARGET", "CARGO_BUILD_TARGET_DIR",
                  "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CC", "CXX", "CFLAGS", "CXXFLAGS", "AR", "CRATE_CC_NO_DEFAULTS", "CXXSTDLIB")


class Refusal(Exception):
    """判不下去：str 是原因，howto 是出路。"""

    def __init__(self, message, howto):
        super().__init__(message)
        self.howto = howto


def broken(name):
    return name in os.environ.get(BREAK_VARIABLE, "").split(",")


def say_red(message, howto, stream=None):
    print(f"  ✗ {message}", file=stream or sys.stdout)
    print(f"  → 怎么办：{howto}", file=stream or sys.stdout)


def home_path(*parts):
    return os.path.join(os.path.expanduser("~"), *parts)


def store_directory():
    return os.environ.get(STORE_VARIABLE) or home_path(".local", "share", "singlefs", "rocksdb-prebuilt")


def environment_file():
    return home_path(".config", "singlefs", "rocksdb-prebuilt.env")


def environment_d_file():
    return home_path(".config", "environment.d", "60-singlefs-rocksdb-prebuilt.conf")


def bashrc_file():
    return home_path(".bashrc")


def today():
    return datetime.datetime.now(zoneinfo.ZoneInfo("Asia/Tokyo")).date().isoformat()


def write_replacing(path, text):
    """同目录临时文件写好再改名换上（不在同一个 inode 上改）；权限照原文件；路径是符号链接的换它指的那一份。"""
    path = os.path.realpath(path)
    directory = os.path.dirname(path)
    os.makedirs(directory, exist_ok=True)
    mode = os.stat(path).st_mode & 0o7777 if os.path.exists(path) else 0o644
    handle, temporary = tempfile.mkstemp(prefix=".rocksdb-prebuilt-", dir=directory)
    with os.fdopen(handle, "w", encoding="utf-8") as stream:
        stream.write(text)
    os.chmod(temporary, mode)
    os.replace(temporary, path)
    with open(path, encoding="utf-8") as stream:
        if stream.read() != text:
            raise Refusal(f"{path} 写完回读对不上", "看那个目录能不能写、有没有别的进程同时在写，再跑一次")


# ── 键 ────────────────────────────────────────────────────────────────────────────────────────

def rocksdb_dependency(root, relative):
    path = os.path.join(root, relative)
    try:
        with open(path, "rb") as handle:
            manifest = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise Refusal(f"读不了 {relative}：{error}", f"{relative} 要是认得出的 TOML，里面有 rocksdb 依赖")
    specification = manifest.get("dependencies", {}).get("rocksdb")
    if specification is None:
        raise Refusal(f"{relative} 里没有 rocksdb 依赖", f"rocksdb 依赖挪走了就改 {SCRIPT_RELATIVE} 的 MANIFESTS")
    if isinstance(specification, str):
        specification = {"version": specification}
    default_features = specification.get("default-features", specification.get("default_features", True))
    return {"version": specification.get("version"), "default-features": bool(default_features),
            "features": sorted(specification.get("features", []))}


def lock_packages(root, relative):
    path = os.path.join(root, relative)
    try:
        with open(path, "rb") as handle:
            packages = tomllib.load(handle).get("package", [])
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise Refusal(f"读不了 {relative}：{error}", f"{relative} 要是 cargo 写的锁文件")
    found = {}
    for package in packages:
        if package.get("name") in KEY_PACKAGES:
            found.setdefault(package["name"], []).append(f"{package.get('version')} {package.get('checksum', '-')}")
    missing = [name for name in KEY_PACKAGES if name not in found]
    if missing:
        raise Refusal(f"{relative} 里没有 {'、'.join(missing)}", "两个工作区都要锁着 rocksdb 与它编 C++ 用的那几个包；先在那个工作区里 cargo generate-lockfile")
    return {name: sorted(found[name]) for name in KEY_PACKAGES}


def first_line_of(command):
    try:
        completed = subprocess.run(command, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    except OSError as error:
        raise Refusal(f"{' '.join(command)} 起不来：{error}", f"装上 {command[0]}，或让它在 PATH 上")
    if completed.returncode != 0:
        raise Refusal(f"{' '.join(command)} 退 {completed.returncode}", f"单独跑 {' '.join(command)} 看它为什么起不来")
    return completed.stdout


def cpu_flags():
    path = os.environ.get(CPUINFO_VARIABLE) or "/proc/cpuinfo"
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.startswith("flags"):
                return set(line.split(":", 1)[1].split())
    return set()


def key_fields(root, workspaces=(0, 1)):
    """这台主机此刻的键的各组成（见文件头「键」）；对不上的拒。workspaces 是 MANIFESTS / LOCKS 里取哪几个工作区（下标），
    entry 只取树里依赖 rocksdb 的那几个（只拷了 crates/ 的仓副本没有 scripts/experiment-apparatus/）；各组成取头一个，两个工作区对得上时与取两个的相同。"""
    dependencies = [rocksdb_dependency(root, MANIFESTS[index]) for index in workspaces]
    if any(dependency != dependencies[0] for dependency in dependencies):
        raise Refusal(f"两个工作区的 rocksdb 依赖写法不同：{MANIFESTS[0]} {dependencies[0]}，{MANIFESTS[1]} {dependencies[1]}",
                      "预编库只有一份，两边链同一份：把两边的 rocksdb 依赖改成同一种写法（版本、默认特性、特性）")
    locks = [lock_packages(root, LOCKS[index]) for index in workspaces]
    if any(lock != locks[0] for lock in locks):
        differing = [name for name in KEY_PACKAGES if locks[0][name] != locks[1][name]]
        raise Refusal(f"两份 Cargo.lock 里 {'、'.join(differing)} 的版本不同", "两边锁到同一版（cargo update -p <包> --precise <版本>），预编库才只要一份")
    flags = cpu_flags()
    lacking = [name for name, flag in CPU_FEATURES if flag not in flags]
    if lacking:
        raise Refusal(f"这台的 CPU 没有 {'、'.join(lacking)}", f"这台不能用预编库这一档指令集：改 {SCRIPT_RELATIVE} 的 CPU_FEATURES 之前先问主 agent")
    fields = {"format": 1, "rocksdb-dependency": dependencies[0], "rustc": first_line_of(["rustc", "-vV"]),
              "cpu-features": [name for name, _ in CPU_FEATURES] + [name for name, flag in OPTIONAL_CPU_FEATURES if flag in flags],
              "profile": BUILD_PROFILE}
    if not broken("key-ignores-lock"):
        fields["lock"] = locks[0]
    if not broken("key-ignores-compiler"):
        compilers = {}
        for name in COMPILERS:
            located = shutil.which(name)
            if located is None:
                raise Refusal(f"PATH 上没有 {name}", f"装上 {name}（librocksdb-sys 经 cc crate 用它编 C++）")
            compilers[name] = f"{os.path.realpath(located)} | {first_line_of([located, '--version']).splitlines()[0].strip()}"
        fields["compilers"] = compilers
    return fields


def key_digest(fields):
    return hashlib.sha256(json.dumps(fields, sort_keys=True, ensure_ascii=False).encode()).hexdigest()[:16]


def key_directory(fields):
    version = fields["lock"]["librocksdb-sys"][0].split()[0] if "lock" in fields else "unknown"
    return os.path.join(store_directory(), f"librocksdb-sys-{version}", key_digest(fields))


def expected_values(fields):
    library = os.path.join(key_directory(fields), "lib")
    return {"ROCKSDB_LIB_DIR": library, "ROCKSDB_STATIC": "1", "SNAPPY_LIB_DIR": library, "SNAPPY_STATIC": "1",
            "ROCKSDB_COMPILE": None, "SNAPPY_COMPILE": None}


def sha256_of(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def store_problem(fields, verify_contents=False):
    """预编库编好了且对得上返回 None，否则返回原因。"""
    directory = key_directory(fields)
    if not os.path.isfile(os.path.join(directory, "complete")):
        return f"{directory} 没编好（没有 complete）"
    try:
        with open(os.path.join(directory, "manifest.json"), encoding="utf-8") as handle:
            manifest = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        return f"{directory}/manifest.json 读不了：{error}"
    if manifest.get("fields") != fields:
        return f"{directory}/manifest.json 记的组成与此刻不同"
    for archive in ARCHIVES:
        path = os.path.join(directory, "lib", archive)
        recorded = manifest.get("archives", {}).get(archive, {})
        if not os.path.isfile(path) or os.path.getsize(path) != recorded.get("bytes"):
            return f"{path} 不在或大小与 manifest.json 不同"
        if verify_contents and sha256_of(path) != recorded.get("sha256"):
            return f"{path} 的 sha256 与 manifest.json 不同"
    return None


# ── 编 ────────────────────────────────────────────────────────────────────────────────────────

def bindgen_arguments(root):
    """仓里 .cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS（串或 {value = …}）；没写交 None。"""
    path = os.path.join(root, ".cargo", "config.toml")
    if not os.path.isfile(path):
        return None
    with open(path, "rb") as handle:
        value = tomllib.load(handle).get("env", {}).get("BINDGEN_EXTRA_CLANG_ARGS")
    return value.get("value") if isinstance(value, dict) else value


def build_environment(root, target_directory, fields):
    environment = {name: value for name, value in os.environ.items()
                   if name not in STRIPPED_NAMES and not name.startswith(STRIPPED_PREFIXES)}
    environment.update({"ROCKSDB_COMPILE": "1", "SNAPPY_COMPILE": "1", "CARGO_TARGET_DIR": target_directory,
                        "RUSTFLAGS": "-C target-feature=" + ",".join(f"+{name}" for name in fields["cpu-features"])})
    arguments = bindgen_arguments(root)
    if arguments:
        environment["BINDGEN_EXTRA_CLANG_ARGS"] = arguments
    return environment


def project_manifest(dependency):
    features = ", ".join(json.dumps(feature) for feature in dependency["features"])
    return ("[package]\nname = \"singlefs-rocksdb-prebuilt\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n"
            "[lib]\npath = \"lib.rs\"\n\n[dependencies]\n"
            f"rocksdb = {{ version = {json.dumps(dependency['version'])}, default-features = {str(dependency['default-features']).lower()}, "
            f"features = [{features}] }}\n\n[profile.release]\nopt-level = {BUILD_PROFILE['opt-level']}\ndebug = {str(BUILD_PROFILE['debug']).lower()}\n\n"
            "[workspace]\n")


def build_once(root, fields, staging):
    """在 staging 里试编、核、拷出两份库；成功交 staging 里编好的那个目录。"""
    project = os.path.join(staging, "project")
    os.makedirs(project)
    with open(os.path.join(project, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write(project_manifest(fields["rocksdb-dependency"]))
    with open(os.path.join(project, "lib.rs"), "w", encoding="utf-8") as handle:
        handle.write("// 只为拉起 librocksdb-sys 的构建脚本编出 librocksdb.a 与 libsnappy.a；本身不用\npub use rocksdb;\n")
    shutil.copyfile(os.path.join(root, "Cargo.lock"), os.path.join(project, "Cargo.lock"))
    target = os.path.join(staging, "target")
    log_path = os.path.join(staging, "build.log")
    started = time.monotonic()
    with open(log_path, "w", encoding="utf-8") as log:
        completed = subprocess.run(["cargo", "build", "--release", "--offline", "--manifest-path", os.path.join(project, "Cargo.toml")],
                                   cwd=project, env=build_environment(root, target, fields), stdout=log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL)
    seconds = round(time.monotonic() - started, 1)
    if completed.returncode != 0:
        kept = os.path.join(store_directory(), "last-failed-build.log")
        shutil.copyfile(log_path, kept)
        raise Refusal(f"试编工程 cargo build 退 {completed.returncode}（日志 {kept}）", f"看 {kept} 末尾的 error，修好再跑 {SCRIPT_RELATIVE} build")
    if not broken("unverified-lock-publishes"):
        built = lock_packages(project, "Cargo.lock")
        if built != fields.get("lock", built):
            differing = [name for name in KEY_PACKAGES if built[name] != fields["lock"][name]]
            raise Refusal(f"试编工程锁到的 {'、'.join(differing)} 与仓里 Cargo.lock 不同，没落库",
                          "试编工程是拷仓根 Cargo.lock 再 --offline 编的；看 registry 缓存里有没有仓里锁的那一版（cargo fetch --locked）")
    outputs = [directory for directory in glob.glob(os.path.join(target, "release", "build", "librocksdb-sys-*"))
               if os.path.isfile(os.path.join(directory, "out", ARCHIVES[0]))]
    if len(outputs) != 1:
        raise Refusal(f"试编工程里编出 {ARCHIVES[0]} 的 librocksdb-sys 构建目录有 {len(outputs)} 个（要恰好 1 个）", f"看 {log_path} 里 librocksdb-sys 的构建脚本跑没跑")
    output_text = open(os.path.join(outputs[0], "output"), encoding="utf-8", errors="replace").read() if os.path.isfile(os.path.join(outputs[0], "output")) else ""
    if "compiling without SSE4.2" in output_text:
        raise Refusal("librocksdb-sys 报「compiling without SSE4.2」：指令集没传到 C++", f"看 {SCRIPT_RELATIVE} 的 build_environment 设的 RUSTFLAGS 进没进构建脚本的 CARGO_CFG_TARGET_FEATURE")
    result = os.path.join(staging, "result")
    os.makedirs(os.path.join(result, "lib"))
    archives = {}
    for archive in ARCHIVES:
        source = os.path.join(outputs[0], "out", archive)
        if not os.path.isfile(source):
            raise Refusal(f"试编工程没编出 {archive}", f"看 {log_path}；snappy 特性关了的话改 {SCRIPT_RELATIVE} 的 ARCHIVES")
        destination = os.path.join(result, "lib", archive)
        shutil.copyfile(source, destination)
        os.chmod(destination, 0o444)
        archives[archive] = {"bytes": os.path.getsize(destination), "sha256": sha256_of(destination)}
    manifest = {"fields": fields, "digest": key_digest(fields), "archives": archives, "build_seconds": seconds, "built_on": today(),
                "build_jobs": os.environ.get("CARGO_BUILD_JOBS", "cargo 默认")}
    with open(os.path.join(result, "manifest.json"), "w", encoding="utf-8") as handle:
        json.dump(manifest, handle, ensure_ascii=False, indent=1, sort_keys=True)
    with open(os.path.join(result, "complete"), "w", encoding="utf-8") as handle:
        handle.write(key_digest(fields) + "\n")
    return result, seconds


def command_build(root):
    fields = key_fields(root)
    directory = key_directory(fields)
    if store_problem(fields) is None:
        print(f"  ✓ 预编库已在：{directory}（不编）")
        return 0
    store = store_directory()
    os.makedirs(store, exist_ok=True)
    lock_handle = open(os.path.join(store, ".build.lock"), "w")
    try:
        if not broken("build-unlocked"):
            try:
                fcntl.flock(lock_handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                print(f"  · 别的进程在编，等它的锁（{store}/.build.lock）", flush=True)
                fcntl.flock(lock_handle, fcntl.LOCK_EX)
            if store_problem(fields) is None:
                print(f"  ✓ 等锁的时候别的进程编好了：{directory}（不编）")
                return 0
        staging = tempfile.mkdtemp(prefix=f".staging-{key_digest(fields)}-", dir=store)
        try:
            result, seconds = build_once(root, fields, staging)
            os.makedirs(os.path.dirname(directory), exist_ok=True)
            if os.path.isdir(directory):
                shutil.rmtree(directory)   # 没有 complete 的半成品，或组成对不上的旧目录（同一把键只会是半成品）
            os.rename(result, directory)
        finally:
            shutil.rmtree(staging, ignore_errors=True)
    finally:
        lock_handle.close()
    problem = store_problem(fields, verify_contents=True)
    if problem:
        raise Refusal(f"换上之后回读：{problem}", f"删掉 {directory} 再跑 {SCRIPT_RELATIVE} build")
    print(f"  ✓ 编好了：{directory}（用时 {seconds} 秒）")
    return 0


# ── 环境来源 ─────────────────────────────────────────────────────────────────────────────────────

def link_values(fields):
    return {name: value for name, value in expected_values(fields).items() if value is not None}


def environment_file_text(fields):
    lines = [f"# {SCRIPT_RELATIVE} install 写的；librocksdb-sys 链这台主机的预编库（键 {key_digest(fields)}）。别手改，改了重跑 install"]
    lines += [f"export {name}={shlex.quote(value)}" for name, value in link_values(fields).items()]
    lines += [f"unset {name}" for name in COMPILE_VARIABLES]
    return "\n".join(lines) + "\n"


def environment_d_text(fields):
    lines = [f"# {SCRIPT_RELATIVE} install 写的；用户级 systemd 管理器起的单元读（键 {key_digest(fields)}）"]
    lines += [f"{name}={value}" for name, value in link_values(fields).items()]
    return "\n".join(lines) + "\n"


def bashrc_block():
    return (f"{BASHRC_BEGIN}（{SCRIPT_RELATIVE} install 写的；放在最前面，ssh 起的非交互命令也读得到）\n"
            "[ -r \"$HOME/.config/singlefs/rocksdb-prebuilt.env\" ] && . \"$HOME/.config/singlefs/rocksdb-prebuilt.env\"\n"
            f"{BASHRC_END}\n")


def bashrc_without_block(text):
    kept, inside = [], False
    for line in text.splitlines(keepends=True):
        if line.startswith(BASHRC_BEGIN):
            inside = True
        elif inside and line.startswith(BASHRC_END):
            inside = False
        elif not inside:
            kept.append(line)
    return "".join(kept)


def parse_assignments(text):
    values = {}
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("unset "):
            values[line.split()[1]] = None
            continue
        words = shlex.split(line[len("export "):] if line.startswith("export ") else line)
        if words and "=" in words[0]:
            name, value = words[0].split("=", 1)
            values[name] = value
    return values


def command_install(root):
    code = command_build(root)
    if code != 0:
        return code
    fields = key_fields(root)
    write_replacing(environment_file(), environment_file_text(fields))
    write_replacing(environment_d_file(), environment_d_text(fields))
    bashrc = bashrc_file()
    original = open(bashrc, encoding="utf-8").read() if os.path.isfile(bashrc) else ""
    backup = bashrc + ".before-singlefs-rocksdb-prebuilt"
    if original and not os.path.exists(backup):
        shutil.copyfile(bashrc, backup)
    write_replacing(bashrc, bashrc_block() + bashrc_without_block(original))
    print(f"  ✓ 写了 {environment_file()}、{environment_d_file()}，{bashrc} 最前面那一段（改前的留在 {backup}）")
    if shutil.which("systemctl"):
        assignments = [f"{name}={value}" for name, value in link_values(fields).items()]
        completed = subprocess.run(["systemctl", "--user", "set-environment", *assignments], capture_output=True, text=True)
        if completed.returncode != 0:
            say_red(f"systemctl --user set-environment 退 {completed.returncode}：{completed.stderr.strip()}",
                    "用户级 systemd 没在跑的话，下次登录时它读 environment.d 那一份；在跑就单独重跑这一步")
    return command_check(root)


# ── check ───────────────────────────────────────────────────────────────────────────────────────

def cargo_configuration_files(root):
    found, directory = [], os.path.abspath(root)
    while True:
        for name in ("config.toml", "config"):
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path):
                found.append(path)
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    cargo_home = os.environ.get("CARGO_HOME") or home_path(".cargo")
    for name in ("config.toml", "config"):
        path = os.path.join(cargo_home, name)
        if os.path.isfile(path) and os.path.realpath(path) not in {os.path.realpath(item) for item in found}:
            found.append(path)
    return found


def compare(values, expected):
    return [f"{name}={values.get(name)!r}（要 {expected[name]!r}）" for name in WATCHED_VARIABLES if values.get(name) != expected[name]]


def command_check(root):
    results = []   # (格名, 红的原因或 None, 出路)；原因写 "not-run:<为什么>" 的是本次没判

    def item(name, problem, howto):
        results.append((name, problem, howto))

    try:
        fields = key_fields(root)
    except Refusal as refusal:
        say_red(f"键算不出：{refusal}", refusal.howto)
        return 1
    item("两个工作区的 rocksdb 依赖与锁对得上、这台 CPU 有预编库用的指令集", None, "")
    expected = expected_values(fields)
    if not broken("missing-store-passes"):
        item("预编库在、对得上", store_problem(fields, verify_contents=True),
             f"bash scripts/hosts/capped.sh <N> python3 {SCRIPT_RELATIVE} build（版本、rustc 或编译器换过就是换了键，要编一次新的）")
    path = environment_file()
    problem = f"{path} 不在" if not os.path.isfile(path) else "；".join(compare(parse_assignments(open(path, encoding="utf-8").read()), expected)) or None
    item(f"{path} 的取值", problem, f"python3 {SCRIPT_RELATIVE} install")
    if not broken("environment-unchecked"):
        mismatched = compare({name: os.environ.get(name) for name in WATCHED_VARIABLES}, expected)
        item("此刻进程环境的取值", "；".join(mismatched) or None,
             f"这个进程是 install 之前起的、或它的父进程没读 {bashrc_file()}：开一个新的登录 shell（或 . {path}）再起；ROCKSDB_COMPILE、SNAPPY_COMPILE 别设")
    bashrc = bashrc_file()
    lines = open(bashrc, encoding="utf-8").read().splitlines() if os.path.isfile(bashrc) else []
    begin = next((index for index, line in enumerate(lines) if line.startswith(BASHRC_BEGIN)), None)
    guard = next((index for index, line in enumerate(lines) if INTERACTIVE_GUARD.match(line)), None)
    problem = (f"{bashrc} 里没有 {BASHRC_BEGIN} 那一段" if begin is None else
               f"{bashrc} 里那一段在第 {begin + 1} 行，交互判断在第 {guard + 1} 行：ssh 起的非交互命令读不到" if guard is not None and guard < begin else None)
    item(f"{bashrc} 最前面 source 取值来源", problem, f"python3 {SCRIPT_RELATIVE} install（它把那一段放到最前面）")
    path = environment_d_file()
    problem = f"{path} 不在" if not os.path.isfile(path) else "；".join(compare(parse_assignments(open(path, encoding="utf-8").read()), expected)) or None
    item(f"{path} 的取值", problem, f"python3 {SCRIPT_RELATIVE} install")
    if shutil.which("systemctl"):
        completed = subprocess.run(["systemctl", "--user", "show-environment"], capture_output=True, text=True)
        if completed.returncode != 0:
            item("用户级 systemd 管理器此刻的环境", f"not-run:systemctl --user show-environment 退 {completed.returncode}", "")
        else:
            item("用户级 systemd 管理器此刻的环境", "；".join(compare(parse_assignments(completed.stdout), expected)) or None,
                 f"python3 {SCRIPT_RELATIVE} install（它跑 systemctl --user set-environment）")
    else:
        item("用户级 systemd 管理器此刻的环境", "not-run:PATH 上没有 systemctl", "")
    written = []
    for configuration in cargo_configuration_files(root):
        try:
            with open(configuration, "rb") as handle:
                table = tomllib.load(handle).get("env", {})
        except (OSError, tomllib.TOMLDecodeError):
            continue
        written += [f"{configuration} 的 [env] {name}" for name in WATCHED_VARIABLES if name in table]
    item("cargo 配置里没写这几个变量", "；".join(written) or None,
         "删掉那几行：写进 cargo 配置的 [env] 会进 crates 变异与广谱变异的键（lib_mutation_units.py 的 toolchain_lines、mutation-prune.py 的 build_environment_lines），取值只走进程环境")
    red = not_run = 0
    for name, problem, howto in results:
        if problem is None:
            print(f"  ✓ {name}")
        elif problem.startswith("not-run:"):
            not_run += 1
            print(f"  · 本次未判 {name}：{problem[len('not-run:'):]}")
        else:
            red += 1
            say_red(f"{name}：{problem}", howto)
    print(f"  {'✗' if red else '✓'} rocksdb-prebuilt check：查了 {len(results)} 项，红 {red} 项，本次未判 {not_run} 项（键 {key_digest(fields)}，预编库 {key_directory(fields)}）")
    if red:
        print(f"  → 怎么办：照上面每一项的出路修，再跑 python3 {SCRIPT_RELATIVE} check")
    return 1 if red else 0


# ── entry ───────────────────────────────────────────────────────────────────────────────────────

def marked_fake_cargo(cargo):
    """cargo（命令名或路径）解到的是带 ENTRY_FAKE_CARGO_MARKER 的脚本时交它的路径，否则 None（文件头「entry」）。
    弄坏开关 entry-skips-unmarked-cargo 下是脚本就交，不看标记。"""
    located = cargo if os.sep in cargo else shutil.which(cargo)
    if not located or not os.path.isfile(located):
        return None
    try:
        with open(located, "rb") as handle:
            head = handle.read(4096)
    except OSError:
        return None
    if head.startswith(b"#!") and (ENTRY_FAKE_CARGO_MARKER.encode() in head or broken("entry-skips-unmarked-cargo")):
        return located
    return None


def manifests_declaring_rocksdb(root):
    """MANIFESTS 里在这棵树上、依赖里有 rocksdb 的那几份（相对路径）；在而读不了的拒。"""
    declared = []
    for relative in MANIFESTS:
        path = os.path.join(root, relative)
        if not os.path.isfile(path):
            continue
        try:
            with open(path, "rb") as handle:
                manifest = tomllib.load(handle)
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise Refusal(f"读不了 {relative}：{error}", f"{relative} 要是认得出的 TOML")
        if "rocksdb" in manifest.get("dependencies", {}):
            declared.append(relative)
    return declared


def command_entry(root, cargo):
    declared = manifests_declaring_rocksdb(root)
    if not declared:
        print(f"  · rocksdb-prebuilt entry：{root} 里 {'、'.join(MANIFESTS)} 都不在或都没有 rocksdb 依赖，编不到 librocksdb-sys，不判")
        return 0
    fake = marked_fake_cargo(cargo)
    if fake is not None:
        print(f"  · rocksdb-prebuilt entry：要起的 cargo 是自证与样本的假 cargo（{fake} 带「{ENTRY_FAKE_CARGO_MARKER}」那一行），不判")
        return 0
    checked, problems = 0, []
    values = {name: os.environ.get(name) for name in WATCHED_VARIABLES}
    if not broken("entry-environment-unchecked"):
        checked += 1
        compiled = [name for name in COMPILE_VARIABLES if values[name] is not None]
        missing = [name for name in LINK_VARIABLES if not values[name]]
        if compiled:
            problems.append((f"此刻进程环境设了 {'、'.join(compiled)}（设了就在编译目录里现编 RocksDB 的 C++）", f"unset {' '.join(compiled)} 再起；{ENTRY_HOWTO}"))
        if missing:
            problems.append((f"此刻进程环境里没有 {'、'.join(missing)}（这个进程是 install 之前起的，或它的父进程没读 {bashrc_file()}）", ENTRY_HOWTO))
    if not problems:
        checked += 1
        try:
            fields = key_fields(root, tuple(MANIFESTS.index(relative) for relative in declared))
        except Refusal as refusal:
            fields = None
            problems.append((f"键算不出：{refusal}", refusal.howto))
        if fields is not None:
            checked += 1
            problem = store_problem(fields)
            if problem:
                problems.append((f"这台的预编库不能用：{problem}", f"bash scripts/hosts/capped.sh <N> python3 {SCRIPT_RELATIVE} install（版本、rustc 或编译器换过就是换了键，要编一次新的）"))
            elif not broken("entry-environment-unchecked"):
                checked += 1
                mismatched = compare(values, expected_values(fields))
                if mismatched:
                    problems.append((f"此刻进程环境的取值与这台的预编库不同：{'；'.join(mismatched)}", ENTRY_HOWTO))
    for message, howto in problems:
        say_red(message, howto)
    if problems:
        print(f"  ✗ rocksdb-prebuilt entry：查了 {checked} 项，红 {len(problems)} 项（{'、'.join(declared)} 依赖 rocksdb）；这一趟不起 cargo，"
              "不然 librocksdb-sys 在编译目录里现编 C++")
        print(f"  → 怎么办：照上面的出路修，再重跑同一条命令；只想看这一步就单跑 python3 {SCRIPT_RELATIVE} entry --root {root}")
        return 1
    print(f"  ✓ rocksdb-prebuilt entry：查了 {checked} 项，进程环境的取值是这台的预编库（{expected_values(fields)['ROCKSDB_LIB_DIR']}）")
    return 0


# ── audit-target ────────────────────────────────────────────────────────────────────────────────

def ledger_path(target):
    return os.path.join(store_directory(), "ledger", hashlib.sha256(os.path.realpath(target).encode()).hexdigest()[:16] + ".tsv")


def residue_in(out_directory):
    if not os.path.isdir(out_directory):
        return []
    found = [entry.name for entry in os.scandir(out_directory) if entry.name in ARCHIVES]
    objects = sum(1 for entry in os.scandir(out_directory) if entry.name.endswith(".o"))
    return found + ([f"{objects} 个 .o"] if objects else [])


def command_audit_target(root, targets):
    try:
        fields = key_fields(root)
    except Refusal as refusal:
        say_red(f"键算不出：{refusal}", refusal.howto)
        return 1
    expected, digest = expected_values(fields), key_digest(fields)
    units = red = 0
    for target in targets:
        fingerprints = sorted({path for pattern in FINGERPRINT_GLOBS for path in glob.glob(os.path.join(target, pattern))})
        ledger = ledger_path(target)
        seen = []
        if os.path.isfile(ledger) and not broken("ledger-ignored"):
            with open(ledger, encoding="utf-8") as handle:
                seen = [line.rstrip("\n").split("\t") for line in handle if line.count("\t") == 2]
        appended = []
        for fingerprint in fingerprints:
            units += 1
            unit_directory = os.path.dirname(fingerprint)
            profile_directory = os.path.dirname(os.path.dirname(unit_directory))
            unit = os.path.relpath(unit_directory, target)
            out_directory = os.path.join(profile_directory, "build", os.path.basename(unit_directory), "out")
            with open(fingerprint, encoding="utf-8") as handle:
                local = json.load(handle).get("local", [])
            values = {name: None for name in WATCHED_VARIABLES}
            tracked = set()
            for entry in local:
                variable = entry.get("RerunIfEnvChanged", {}) if isinstance(entry, dict) else {}
                if variable.get("var") in values:
                    values[variable["var"]] = variable.get("val")
                    tracked.add(variable["var"])
            observed = json.dumps(values, sort_keys=True)
            problems = []
            if not broken("fingerprint-ignored"):
                mismatched = compare(values, expected)
                if mismatched:
                    problems.append("构建脚本那一次拿到的取值不是这台的预编库：" + "；".join(mismatched))
            if not broken("residue-ignored"):
                residue = residue_in(out_directory)
                if residue:
                    problems.append(f"{out_directory} 里有现编过的 C++（{'、'.join(residue)}）：这份编译目录里有入口没拿到预编库的取值")
            if not broken("ledger-ignored"):
                earlier = sorted({row[2] for row in seen if row[0] == digest and row[1] == unit and row[2] != observed})
                if earlier:
                    problems.append(f"同一把键（{digest}）下这份编译目录见过别的取值：{'；'.join(earlier)}，此刻 {observed}")
                if not any(row[0] == digest and row[1] == unit and row[2] == observed for row in seen):
                    appended.append(f"{digest}\t{unit}\t{observed}\n")
            if problems:
                red += 1
                for problem in problems:
                    print(f"  ✗ {unit}：{problem}")  # gate-lint:detail
            else:
                print(f"  ✓ {unit}：链预编库，没有现编的残留")
        if appended:
            os.makedirs(os.path.dirname(ledger), exist_ok=True)
            with open(ledger, "a", encoding="utf-8") as handle:
                handle.writelines(appended)
    if units == 0:
        print(f"  · rocksdb-prebuilt audit-target：{len(targets)} 份编译目录里一份 librocksdb-sys 构建脚本记录都没有（查了 0 份），本次未判")
        return EXIT_NOT_RUN
    print(f"  {'✗' if red else '✓'} rocksdb-prebuilt audit-target：{len(targets)} 份编译目录、查了 {units} 份 librocksdb-sys 构建脚本记录，红 {red} 份（键 {digest}）")
    if red:
        print(f"  → 怎么办：先跑 python3 {SCRIPT_RELATIVE} check 修好入口的环境，再删红的那几份的 <构建档>/build/librocksdb-sys-<号> 与 <构建档>/.fingerprint/librocksdb-sys-<号>"
              f"（没有 cargo 在用这份编译目录时删），下一次编译重跑构建脚本、链预编库；见过的取值记在 {store_directory()}/ledger/，删了编译目录一并删它那一份")
    return 1 if red else 0


# ── 自证 ─────────────────────────────────────────────────────────────────────────────────────────

FAKE_TOOL = """import os, sys
name = os.path.basename(sys.argv[0])
if name == "rustc":
    print(os.environ.get("FAKE_RUSTC_TEXT", "rustc 9.9.9 (selftest)\\nhost: x86_64-unknown-linux-gnu"))
elif name in ("c++", "cc", "ar"):
    print(os.environ.get("FAKE_COMPILER_VERSION", "fake-compiler 1.0") + " " + name)
elif name == "systemctl":
    state = os.environ["FAKE_SYSTEMCTL_STATE"]
    if sys.argv[1:3] == ["--user", "show-environment"]:
        print(open(state).read() if os.path.exists(state) else "", end="")
    elif sys.argv[1:3] == ["--user", "set-environment"]:
        with open(state, "a") as handle:
            handle.write("".join(item + "\\n" for item in sys.argv[3:]))
elif name == "cargo":
    import shutil, time
    with open(os.environ["FAKE_CARGO_COUNT"], "a") as handle:
        handle.write(os.environ.get("RUSTFLAGS", "-") + "\\n")
    time.sleep(float(os.environ.get("FAKE_CARGO_SLEEP", "0")))
    manifest = sys.argv[sys.argv.index("--manifest-path") + 1]
    project = os.path.dirname(manifest)
    if os.environ.get("FAKE_CARGO_LOCK_DRIFT"):
        text = open(os.path.join(project, "Cargo.lock")).read().replace('version = "2.1.0+zstd.1.5.7"', 'version = "2.0.0"')
        open(os.path.join(project, "Cargo.lock"), "w").write(text)
    assert os.environ.get("ROCKSDB_COMPILE") == "1" and "ROCKSDB_LIB_DIR" not in os.environ
    out = os.path.join(os.environ["CARGO_TARGET_DIR"], "release", "build", "librocksdb-sys-0123456789abcdef", "out")
    os.makedirs(out)
    for archive in ("librocksdb.a", "libsnappy.a"):
        open(os.path.join(out, archive), "wb").write(b"!<arch>\\n" + archive.encode())
    open(os.path.join(os.path.dirname(out), "output"), "w").write("cargo:rustc-link-lib=static=rocksdb\\n")
    sys.exit(int(os.environ.get("FAKE_CARGO_EXIT", "0")))
"""

SELFTEST_LOCK = """version = 4

[[package]]
name = "bzip2-sys"
version = "0.1.13+1.0.8"
checksum = "b1"

[[package]]
name = "cc"
version = "1.5.1"
checksum = "c1"

[[package]]
name = "libz-sys"
version = "1.1.29"
checksum = "z1"

[[package]]
name = "librocksdb-sys"
version = "0.19.0+11.8.1"
checksum = "r1"

[[package]]
name = "lz4-sys"
version = "1.11.1+lz4-1.10.0"
checksum = "l1"

[[package]]
name = "rocksdb"
version = "0.25.0"
checksum = "k1"

[[package]]
name = "zstd-sys"
version = "2.1.0+zstd.1.5.7"
checksum = "s1"
"""


class Selftest:
    def __init__(self, scratch):
        self.scratch = scratch
        self.failures = []
        self.cells = 0
        self.root = os.path.join(scratch, "tree")
        self.fake = os.path.join(scratch, "fake-bin")
        os.makedirs(self.fake)
        for name in ("rustc", "cargo", "c++", "cc", "ar", "systemctl"):
            path = os.path.join(self.fake, name)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(f"#!{sys.executable}\n{FAKE_TOOL}")
            os.chmod(path, 0o755)
        for relative, text in (("Cargo.lock", SELFTEST_LOCK), ("scripts/experiment-apparatus/Cargo.lock", SELFTEST_LOCK),
                               ("crates/singlefs-checker-tier/Cargo.toml", "[package]\nname = \"x\"\n[dependencies]\nrocksdb = { version = \"=0.25.0\", optional = true }\n"),
                               ("scripts/experiment-apparatus/Cargo.toml", "[package]\nname = \"y\"\n[dependencies.rocksdb]\nversion = \"=0.25.0\"\noptional = true\n"),
                               (".cargo/config.toml", "[env]\nBINDGEN_EXTRA_CLANG_ARGS = \"-I/nowhere\"\n"),
                               ("cpuinfo", "processor : 0\nflags : fpu sse2 sse4_1 sse4_2 pclmulqdq avx2\n")):
            path = os.path.join(self.root, relative)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
        self.home = os.path.join(scratch, "home")
        os.makedirs(self.home)
        with open(os.path.join(self.home, ".bashrc"), "w", encoding="utf-8") as handle:
            handle.write("# 用户自己的\ncase $- in\n    *i*) ;;\n      *) return;;\nesac\nalias ll='ls -l'\n")
        self.base = {name: value for name, value in os.environ.items() if name not in WATCHED_VARIABLES and name != BREAK_VARIABLE}
        self.base.update({"PATH": self.fake + os.pathsep + os.environ.get("PATH", ""), "HOME": self.home, "CARGO_HOME": os.path.join(self.home, ".cargo"),
                          STORE_VARIABLE: os.path.join(scratch, "store"), CPUINFO_VARIABLE: os.path.join(self.root, "cpuinfo"),
                          "FAKE_CARGO_COUNT": os.path.join(scratch, "cargo-count"), "FAKE_SYSTEMCTL_STATE": os.path.join(scratch, "systemd-environment")})

    def run(self, arguments, **overrides):
        environment = dict(self.base)
        environment.update({name: value for name, value in overrides.items() if value is not None})
        for name, value in overrides.items():
            if value is None:
                environment.pop(name, None)
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), *arguments, "--root", self.root],
                                   capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL)
        return completed.returncode, completed.stdout + completed.stderr

    def check(self, name, condition, detail=""):
        self.cells += 1
        if not condition:
            self.failures.append(name)
            print(f"  ✗ 自证 {name}：{detail.strip()[-1500:]}")  # gate-lint:detail
        else:
            print(f"  ✓ 自证 {name}")

    def builds(self):
        path = self.base["FAKE_CARGO_COUNT"]
        return sum(1 for _ in open(path)) if os.path.exists(path) else 0

    def key_line(self, **overrides):
        code, output = self.run(["key"], **overrides)
        return next((line for line in output.splitlines() if line.startswith("key=")), f"退 {code}：{output}")


def write_fingerprint(target, profile, unit_hash, values):
    directory = os.path.join(target, profile, ".fingerprint", f"librocksdb-sys-{unit_hash}")
    os.makedirs(directory, exist_ok=True)
    local = [{"RerunIfChanged": {"output": "x", "paths": ["rocksdb/"]}}]
    local += [{"RerunIfEnvChanged": {"var": name, "val": values.get(name)}} for name in WATCHED_VARIABLES]
    with open(os.path.join(directory, "run-build-script-build-script-build.json"), "w", encoding="utf-8") as handle:
        json.dump({"rustc": 1, "local": local}, handle)
    os.makedirs(os.path.join(target, profile, "build", f"librocksdb-sys-{unit_hash}", "out"), exist_ok=True)


def run_selftest():
    if os.environ.get(BREAK_VARIABLE):
        unknown = [name for name in os.environ[BREAK_VARIABLE].split(",") if name not in BREAK_SWITCHES]
        if unknown:
            say_red(f"认不出的弄坏开关 {'、'.join(unknown)}", f"认得的：{'、'.join(BREAK_SWITCHES)}")
            return 2
    scratch = tempfile.mkdtemp(prefix="rocksdb-prebuilt-selftest-")
    try:
        selftest = Selftest(scratch)
        breaks = os.environ.get(BREAK_VARIABLE)
        child = {BREAK_VARIABLE: breaks} if breaks else {}

        base_key = selftest.key_line(**child)
        selftest.check("换了 C++ 编译器键就变", base_key.startswith("key=") and selftest.key_line(FAKE_COMPILER_VERSION="fake-compiler 2.0", **child) != base_key, base_key)
        drifted = os.path.join(scratch, "drifted")
        shutil.copytree(selftest.root, drifted)
        for relative in LOCKS:
            path = os.path.join(drifted, relative)
            text = open(path).read().replace('version = "0.19.0+11.8.1"', 'version = "0.20.0+11.9.0"')
            open(path, "w").write(text)
        drifted_key = next((line for line in subprocess.run([sys.executable, os.path.realpath(__file__), "key", "--root", drifted], capture_output=True,
                                                            text=True, env={**selftest.base, **child}).stdout.splitlines() if line.startswith("key=")), "")
        selftest.check("换了 librocksdb-sys 版本键就变", drifted_key.startswith("key=") and drifted_key != base_key, f"{base_key} / {drifted_key}")
        selftest.check("换了 rustc 键就变", selftest.key_line(FAKE_RUSTC_TEXT="rustc 9.9.10 (selftest)", **child) != base_key, base_key)
        with open(os.path.join(scratch, "no-sse42"), "w", encoding="utf-8") as handle:
            handle.write("flags : fpu sse2\n")
        code, output = selftest.run(["key"], **{CPUINFO_VARIABLE: os.path.join(scratch, "no-sse42")}, **child)
        selftest.check("这台 CPU 缺指令集就拒", code == 1 and "sse4.2" in output, output)
        with open(os.path.join(scratch, "no-avx2"), "w", encoding="utf-8") as handle:
            handle.write("flags : fpu sse2 sse4_1 sse4_2 pclmulqdq\n")
        selftest.check("这台有没有 AVX2 键不同", selftest.key_line(**{CPUINFO_VARIABLE: os.path.join(scratch, "no-avx2")}, **child) not in (base_key, ""), base_key)

        code, output = selftest.run(["check"], **child)
        selftest.check("预编库不在 check 判红", code == 1 and "预编库在、对得上" in output and "✗" in output, output)
        code, output = selftest.run(["env"], **child)
        selftest.check("预编库不在 env 退 1、标准输出不给 export", code == 1 and "export" not in output, output)

        code, output = selftest.run(["build"], FAKE_CARGO_LOCK_DRIFT="1", **child)
        selftest.check("试编工程的依赖版本与仓里对不上不落库", code == 1 and "zstd-sys" in output and not glob.glob(os.path.join(scratch, "store", "librocksdb-sys-*", "*", "complete")), output)
        code, output = selftest.run(["build"], FAKE_CARGO_EXIT="101", **child)
        selftest.check("试编失败不落库、不留临时目录", code == 1 and not glob.glob(os.path.join(scratch, "store", "librocksdb-sys-*", "*", "complete"))
                       and not glob.glob(os.path.join(scratch, "store", ".staging-*")), output)

        open(selftest.base["FAKE_CARGO_COUNT"], "w").close()
        environment = {**selftest.base, **child, "FAKE_CARGO_SLEEP": "1.5"}
        command = [sys.executable, os.path.realpath(__file__), "build", "--root", selftest.root]
        racers = [subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=environment) for _ in range(2)]
        outcomes = [(racer.communicate()[0], racer.returncode) for racer in racers]
        selftest.check("两个 build 同时来只编一次", [code for _, code in outcomes] == [0, 0] and selftest.builds() == 1,
                       f"cargo 起了 {selftest.builds()} 次；" + " | ".join(text for text, _ in outcomes))
        code, output = selftest.run(["build"], **child)
        selftest.check("编好了再 build 不编", code == 0 and selftest.builds() == 1 and "不编" in output, output)
        rustflags = open(selftest.base["FAKE_CARGO_COUNT"]).read()
        selftest.check("试编交给 C++ 的指令集是键里的那几样", "+sse4.2" in rustflags and "+pclmulqdq" in rustflags and "+avx2" in rustflags and "+bmi1" not in rustflags, rustflags)

        code, output = selftest.run(["env"], **child)
        exports = parse_assignments(output)
        library = exports.get("ROCKSDB_LIB_DIR", "")
        selftest.check("env 打四个取值，指到编好的库", code == 0 and set(exports) == set(LINK_VARIABLES) and os.path.isfile(os.path.join(library, "librocksdb.a")), output)

        code, output = selftest.run(["install"], **child, **{name: value for name, value in exports.items()})
        bashrc = open(os.path.join(selftest.home, ".bashrc")).read()
        selftest.check("install 写齐三处来源、bashrc 那一段在交互判断之前、用户原文留着", code == 0 and bashrc.startswith(BASHRC_BEGIN) and "alias ll" in bashrc
                       and os.path.isfile(os.path.join(selftest.home, ".config", "environment.d", "60-singlefs-rocksdb-prebuilt.conf")), output)
        code, output = selftest.run(["install"], **child, **exports)
        selftest.check("install 重跑不叠第二段", code == 0 and open(os.path.join(selftest.home, ".bashrc")).read().count(BASHRC_BEGIN) == 1, output)
        code, output = selftest.run(["check"], **child, **exports)
        selftest.check("装好之后 check 全绿", code == 0 and "红 0 项" in output, output)
        code, output = selftest.run(["check"], **child, **{**exports, "ROCKSDB_LIB_DIR": None})
        selftest.check("进程环境里的取值与预编库不同 check 判红", code == 1 and "此刻进程环境的取值" in output, output)
        code, output = selftest.run(["check"], **child, **{**exports, "ROCKSDB_COMPILE": "1"})
        selftest.check("进程环境里设了 ROCKSDB_COMPILE check 判红", code == 1 and "ROCKSDB_COMPILE" in output, output)
        code, output = selftest.run(["check"], FAKE_COMPILER_VERSION="fake-compiler 3.0", **child, **exports)
        selftest.check("换了编译器之后旧的取值 check 判红（要编新键）", code == 1 and "预编库在、对得上" in output, output)
        os.makedirs(os.path.join(selftest.home, ".cargo"), exist_ok=True)
        with open(os.path.join(selftest.home, ".cargo", "config.toml"), "w") as handle:
            handle.write(f"[env]\nROCKSDB_LIB_DIR = \"{library}\"\n")
        code, output = selftest.run(["check"], **child, **exports)
        selftest.check("cargo 配置的 [env] 写了这几个变量 check 判红", code == 1 and "进 crates 变异与广谱变异的键" in output, output)
        os.remove(os.path.join(selftest.home, ".cargo", "config.toml"))

        # entry（文件头「entry」）：PATH 最前面的假 cargo（FAKE_TOOL）不带标记，照判
        bare = {name: None for name in WATCHED_VARIABLES}
        code, output = selftest.run(["entry"], **child, **bare)
        selftest.check("entry：进程环境没有取值判红、不往下算键、出路是开新会话或 eval env", code == 1 and "此刻进程环境里没有" in output and "开新会话" in output
                       and 'eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"' in output and "查了 1 项" in output, output)
        code, output = selftest.run(["entry"], **child, **{**exports, "ROCKSDB_LIB_DIR": "/elsewhere/lib"})
        selftest.check("entry：取值与预编库不同判红", code == 1 and "与这台的预编库不同" in output and "/elsewhere/lib" in output, output)
        code, output = selftest.run(["entry"], **child, **{**exports, "ROCKSDB_COMPILE": "1"})
        selftest.check("entry：设了 ROCKSDB_COMPILE 判红", code == 1 and "设了 ROCKSDB_COMPILE" in output, output)
        code, output = selftest.run(["entry"], **child, **exports)
        selftest.check("entry：装好、取值对得上判绿", code == 0 and "rocksdb-prebuilt entry：查了 4 项" in output, output)
        marked_directory = os.path.join(scratch, "marked-bin")
        os.makedirs(marked_directory)
        marked = os.path.join(marked_directory, "cargo")
        with open(marked, "w", encoding="utf-8") as handle:
            handle.write(f"#!/bin/sh\n# {ENTRY_FAKE_CARGO_MARKER}\nexit 0\n")
        os.chmod(marked, 0o755)
        code, output = selftest.run(["entry"], **child, **bare, PATH=marked_directory + os.pathsep + selftest.base["PATH"])
        selftest.check("entry：PATH 最前面是带标记的假 cargo 不判", code == 0 and "不判" in output and marked in output, output)
        code, output = selftest.run(["entry", "--cargo", marked], **child, **bare)
        selftest.check("entry：--cargo 点名的假 cargo 带标记不判", code == 0 and "不判" in output and marked in output, output)
        code, output = selftest.run(["entry", "--cargo", os.path.join(selftest.fake, "cargo")], **child, **bare)
        selftest.check("entry：--cargo 点名的假 cargo 不带标记照判", code == 1 and "此刻进程环境里没有" in output, output)
        plain = os.path.join(scratch, "plain-tree")
        os.makedirs(os.path.join(plain, "crates", "singlefs-checker-tier"))
        with open(os.path.join(plain, "crates", "singlefs-checker-tier", "Cargo.toml"), "w", encoding="utf-8") as handle:
            handle.write('[package]\nname = "x"\n[dependencies]\nserde = "1"\n')
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "entry", "--root", plain], capture_output=True, text=True,
                                   env={**selftest.base, **child}, stdin=subprocess.DEVNULL)
        selftest.check("entry：树里没有 rocksdb 依赖不判", completed.returncode == 0 and "编不到 librocksdb-sys" in completed.stdout, completed.stdout + completed.stderr)
        crates_only = os.path.join(scratch, "crates-only-copy")
        shutil.copytree(selftest.root, crates_only)
        shutil.rmtree(os.path.join(crates_only, "scripts"))
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "entry", "--root", crates_only], capture_output=True, text=True,
                                   env={**selftest.base, **child, **exports}, stdin=subprocess.DEVNULL)
        selftest.check("entry：只拷了 crates/ 的仓副本照判、键与两个工作区都在时相同", completed.returncode == 0 and "查了 4 项" in completed.stdout,
                       completed.stdout + completed.stderr)

        target = os.path.join(scratch, "target")
        good = {name: exports.get(name) for name in WATCHED_VARIABLES}
        code, output = selftest.run(["audit-target", target], **child, **exports)
        selftest.check("编译目录里没有 librocksdb-sys 记录 audit-target 退 77", code == EXIT_NOT_RUN and "查了 0 份" in output, output)
        write_fingerprint(target, "debug", "aaaa", good)
        write_fingerprint(target, "release", "bbbb", good)
        code, output = selftest.run(["audit-target", target], **child, **exports)
        selftest.check("debug 与 release 都链预编库 audit-target 全绿", code == 0 and "查了 2 份" in output and "红 0 份" in output, output)
        write_fingerprint(target, "debug", "cccc", {})
        code, output = selftest.run(["audit-target", target], **child, **exports)
        selftest.check("编译目录没拿到预编库的取值 audit-target 判红", code == 1 and "debug/.fingerprint/librocksdb-sys-cccc" in output and "不是这台的预编库" in output, output)
        shutil.rmtree(os.path.join(target, "debug", ".fingerprint", "librocksdb-sys-cccc"))
        shutil.rmtree(os.path.join(target, "debug", "build", "librocksdb-sys-cccc"))
        # 同一编译目录两次取值不同：没带取值的入口编过一次（现编 C++，out/ 里留下 .a 与 .o），带取值的入口再编一次（指纹又是预编库的取值）
        write_fingerprint(target, "debug", "aaaa", {})
        out = os.path.join(target, "debug", "build", "librocksdb-sys-aaaa", "out")
        for name in ("librocksdb.a", "db_impl.o"):
            open(os.path.join(out, name), "w").close()
        write_fingerprint(target, "debug", "aaaa", good)
        code, output = selftest.run(["audit-target", target], **child, **exports)
        selftest.check("同一编译目录两次取值不同（现编过又换回预编）audit-target 判红", code == 1 and "现编过的 C++" in output, output)
        for name in ("librocksdb.a", "db_impl.o"):
            os.remove(os.path.join(out, name))
        write_fingerprint(target, "release", "bbbb", {**good, "SNAPPY_STATIC": None})
        selftest.run(["audit-target", target], **child, **exports)
        write_fingerprint(target, "release", "bbbb", good)
        code, output = selftest.run(["audit-target", target], **child, **exports)
        selftest.check("同一把键下同一编译目录见过两种取值 audit-target 判红", code == 1 and "见过别的取值" in output and "release/.fingerprint/librocksdb-sys-bbbb" in output, output)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    total = selftest.cells
    if selftest.failures:
        print(f"  ✗ rocksdb-prebuilt 自证：{total} 格里红 {len(selftest.failures)} 格（{'、'.join(selftest.failures)}）")
        print(f"  → 怎么办：照红的那一格修 {SCRIPT_RELATIVE}；弄坏开关打开时红在文件头写的那一格是对的")
        return 1
    print(f"  ✓ rocksdb-prebuilt 自证：{total} 格都过")
    return 0


# ── 入口 ─────────────────────────────────────────────────────────────────────────────────────────

USAGE = f"用法：{SCRIPT_RELATIVE} key|env|build|install|check [--root R] | entry [--root R] [--cargo C] | audit-target <编译目录>… [--root R] | --selftest"


def main(argv):
    if argv == ["--selftest"]:
        return run_selftest()
    if os.environ.get(BREAK_VARIABLE) and STORE_VARIABLE not in os.environ:
        say_red(f"{BREAK_VARIABLE} 只给 --selftest 用（自证起的子进程都设了 {STORE_VARIABLE}）", f"拿掉 {BREAK_VARIABLE} 再跑")
        return 2
    root, rest = REPOSITORY_ROOT, []
    index = 0
    while index < len(argv):
        if argv[index] == "--root" and index + 1 < len(argv):
            root = os.path.abspath(argv[index + 1])
            index += 2
            continue
        rest.append(argv[index])
        index += 1
    if not rest:
        say_red("没给子命令", USAGE)
        return 2
    command, arguments = rest[0], rest[1:]
    try:
        if command == "audit-target":
            if not arguments:
                say_red("audit-target 没给编译目录", USAGE)
                return 2
            return command_audit_target(root, arguments)
        if command == "entry":
            if arguments and (len(arguments) != 2 or arguments[0] != "--cargo" or not arguments[1]):
                say_red(f"entry 认不出参数 {' '.join(arguments)}", USAGE)
                return 2
            return command_entry(root, arguments[1] if arguments else "cargo")
        if arguments:
            say_red(f"认不出参数 {' '.join(arguments)}", USAGE)
            return 2
        if command == "key":
            fields = key_fields(root)
            print(json.dumps(fields, ensure_ascii=False, indent=1, sort_keys=True))
            print(f"key={key_digest(fields)}")
            print(f"directory={key_directory(fields)}")
            return 0
        if command == "env":
            fields = key_fields(root)
            problem = store_problem(fields)
            if problem:
                say_red(f"预编库不能用：{problem}", f"bash scripts/hosts/capped.sh <N> python3 {SCRIPT_RELATIVE} build", stream=sys.stderr)
                return 1
            print("".join(f"export {name}={shlex.quote(value)}\n" for name, value in link_values(fields).items()), end="")
            return 0
        if command == "build":
            return command_build(root)
        if command == "install":
            return command_install(root)
        if command == "check":
            return command_check(root)
    except Refusal as refusal:
        say_red(str(refusal), refusal.howto)
        return 1
    say_red(f"认不出子命令 {command}", USAGE)
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
