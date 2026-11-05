#!/usr/bin/env python3
# admission: always 每次调都现判此刻的多机配置与这一台网卡上的地址，上一次取到的地址不替这一次作保
# run-condition: command bash ip
"""多机配置的主机数组（HOSTS）里哪一项是这一台、哪一项是第二台：python 一侧的取法，只这一处（shell 一侧是 scripts/hosts/peer-host-lib.sh 的 peer_host_resolve_addresses）。
不另写一份判法：起配置判法 crash-point-replay-shard-configuration-check.sh --emit-addresses [--configuration <配置文件>]（整份判、不连任何一台、不看双机开关；
判法文件头「主机数组」），读它打的 THIS_HOST_ADDRESS 与 PEER_HOST_ADDRESS。判法 source peer-host-lib.sh 调同一个函数，两侧认法逐字相同：
照这一台网卡上的地址认自己（恰好一项），另一项是第二台（此刻只认两台），没写 HOSTS、写着判法认不出的键、认不出自己都判不过。只用标准库，不依赖 task-server/。

按台的三个键（BY_HOST_KEYS：CARGO_BIN_DIRECTORY_BY_HOST、BUILD_ROOT_BY_HOST、SLICE_MEMORY_CAP_BY_HOST，意思与写法在仓根 multi-host.env.example）：
  值是空白分隔的 <地址>=<值>，每项从第一个 = 切；HOSTS 里每一台恰好一项、每个地址都在 HOSTS 里，少了、多了、同一台写两遍、项里没有 = 或值是空的都判写坏。
  要哪一台的就按那一台在 HOSTS 里的地址取（经 ssh 去对面取对面那一项，本机用取这一台那一项；这一台照 resolve 认）。整份配置的判法（三个键必写、每台恰好一项）
  在 scripts/crash/crash-point-replay-shard-configuration-check.sh；python 一侧读配置的一方（task-server）照 by_host_values 切、再判一遍，shell 一侧只按地址取那一项
  （scripts/hosts/peer-host-lib.sh 的 peer_host_value_for_host，判法过了才读）；值的写法（绝对路径、上限的单位）由用它的一方判。

用法：
  python  按文件路径导入（importlib.util.spec_from_file_location），调
            peer_ssh_address(configuration_path, environment=None) → (第二台的地址或 None, 取不到的原因连出路)
            resolve(configuration_path=None, environment=None) → Addresses(this, peer)；取不到抛 AddressesUnavailable（ValueError 的子类，str 是原因）
            by_host_values(text, hosts, key) → {地址: 值}；写坏抛 ByHostValueUnavailable（ValueError 的子类，str 是原因，点名键与哪一项）
            configured_by_host(configuration_path, key) → {地址: 值}：照配置里的 HOSTS 与那个键切，读不到或写坏抛 ByHostValueUnavailable
          configuration_path 给了就判那一份（--configuration）；None 照判法的找法（SINGLEFS_MULTI_HOST_CONFIG，没设取主工作树根的 multi-host.env）；
          environment 是起判法的环境（None 是此刻的 os.environ）。
  shell   python3 scripts/hosts/multi_host_addresses.py [--configuration <配置文件>] peer|this：打一行那个地址；取不到退 1，标准错误写原因与出路
  自证    python3 scripts/hosts/multi_host_addresses.py --selftest：临时目录里的假配置（127.0.0.1 在每一台的网卡上、192.0.2.x 是文档用的地址段），起真的判法；
          MULTI_HOST_ADDRESSES_BREAK=<项> 时必须判红
退出码：0 打出了；1 取不到；2 用法错（含弄坏开关设在自证之外）。

弄坏开关 MULTI_HOST_ADDRESSES_BREAK=<项>（只在 --selftest 与它起的子进程里认：自证设 MULTI_HOST_ADDRESSES_SELFTEST=1；别处设了退 2、点名它只在自证里认）：
  MULTI_HOST_ADDRESSES_BREAK=first-item-taken  不经判法，直接把配置里 HOSTS 的头一项当第二台（写死次序的样子）
      「✗ multi_host_addresses 自证：双机关着也照取（判法 --emit-addresses 不看开关）」（那一份自己写在头一项）、
      「✗ multi_host_addresses 自证：这一台不在数组里：取不到、点名判法的原因」
  MULTI_HOST_ADDRESSES_BREAK=by-host-missing-unchecked  by_host_values 不核 HOSTS 里每一台都有一项（少了照收）
      「✗ multi_host_addresses 自证：按台的键少了一台：判写坏、点名缺的那一台」
  MULTI_HOST_ADDRESSES_BREAK=by-host-outside-unchecked  by_host_values 不核每个地址都在 HOSTS 里（多了照收）
      「✗ multi_host_addresses 自证：按台的键多一项不在 HOSTS 里的地址：判写坏、点名那一项」
  （task-server 的弄坏开关 by-host-missing-accepted、by-host-outside-accepted 经 by_host_values 的 environment 参数设这两个，不经进程环境：
    task-server/task_server_lib.py 的 configured_by_host）
  MULTI_HOST_ADDRESSES_BREAK=by-host-split-at-last-equals  by_host_values 每项从最后一个 = 切（值里自己带 = 的切错）
      「✗ multi_host_addresses 自证：按台取值：各台取各台那一项，值里的 = 照留」
"""
import collections
import os
import shlex
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
CONFIGURATION_CHECK = os.path.join(REPOSITORY_ROOT, "scripts", "crash", "crash-point-replay-shard-configuration-check.sh")
BREAK_VARIABLE = "MULTI_HOST_ADDRESSES_BREAK"
SELFTEST_VARIABLE = "MULTI_HOST_ADDRESSES_SELFTEST"
KNOWN_BREAKS = ("first-item-taken", "by-host-missing-unchecked", "by-host-outside-unchecked", "by-host-split-at-last-equals")
HOSTS_KEY = "HOSTS"
BY_HOST_KEYS = ("CARGO_BIN_DIRECTORY_BY_HOST", "BUILD_ROOT_BY_HOST", "SLICE_MEMORY_CAP_BY_HOST")   # 按台的三个键（文件头「按台的三个键」）
BY_HOST_HOWTO = "写成 <键>=<地址>=<值> <地址>=<值>（HOSTS 里每一台恰好一项、地址都在 HOSTS 里），照仓根 multi-host.env.example 改"
CHECK_SECONDS = 60
HOWTO = "单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses 看原因；多机配置照仓根 multi-host.env.example 写 HOSTS=<各台有线直连段的 IP，空格分隔>"
Addresses = collections.namedtuple("Addresses", "this peer")


class AddressesUnavailable(ValueError):
    """取不到主机数组里的地址：str 是原因。"""


class ByHostValueUnavailable(ValueError):
    """按台的键读不到或写坏了：str 是原因（点名键与哪一项）。"""


def broken(item, environment):
    return item in (environment.get(BREAK_VARIABLE) or "").split(",")


def by_host_values(text, hosts, key, environment=None):
    """按台的键的整值 → {地址: 值}（文件头「按台的三个键」）：空白分隔的 <地址>=<值>，每项从第一个 = 切；hosts 是 HOSTS 的各项。
    少一台、多一个不在 HOSTS 里的地址、同一台写两遍、项里没有 = 或值是空的抛 ByHostValueUnavailable。environment 是读弄坏开关的那一份（None 是此刻的 os.environ）：
    by-host-missing-unchecked 下少了台照收，by-host-outside-unchecked 下多了不在 HOSTS 里的照收，by-host-split-at-last-equals 下从最后一个 = 切。"""
    environment = os.environ if environment is None else environment
    values = {}
    for item in (text or "").split():
        if "=" not in item:
            raise ByHostValueUnavailable(f"{key} 的「{item}」没有 =（每项写 <地址>=<值>）；{BY_HOST_HOWTO}")
        address, value = item.rsplit("=", 1) if broken("by-host-split-at-last-equals", environment) else item.split("=", 1)
        if not address or not value:
            raise ByHostValueUnavailable(f"{key} 的「{item}」地址或值是空的；{BY_HOST_HOWTO}")
        if address in values:
            raise ByHostValueUnavailable(f"{key} 里 {address} 写了两遍；{BY_HOST_HOWTO}")
        values[address] = value
    outside = [address for address in values if address not in hosts]
    if outside and not broken("by-host-outside-unchecked", environment):
        raise ByHostValueUnavailable(f"{key} 里 {'、'.join(outside)} 不在 {HOSTS_KEY} 里（{HOSTS_KEY} 是「{' '.join(hosts)}」）；{BY_HOST_HOWTO}")
    missing = [address for address in hosts if address not in values]
    if missing and not broken("by-host-missing-unchecked", environment):
        raise ByHostValueUnavailable(f"{key} 里少了 {'、'.join(missing)} 那一台（{HOSTS_KEY} 里每一台恰好一项）；{BY_HOST_HOWTO}")
    return values


def configured_by_host(configuration_path, key, environment=None):
    """多机配置里一个按台的键 → {地址: 值}：照那份配置的 HOSTS 与这个键（以「名字=」起头的最后一行，去一层成对引号）切；
    配置读不了、没写 HOSTS 或这个键、写坏都抛 ByHostValueUnavailable。"""
    try:
        hosts_text, text = configuration_value(configuration_path, HOSTS_KEY), configuration_value(configuration_path, key)
    except OSError as error:
        raise ByHostValueUnavailable(f"多机配置 {configuration_path} 读不了（{error}）") from error
    if not (hosts_text or "").split():
        raise ByHostValueUnavailable(f"多机配置 {configuration_path} 里没写 {HOSTS_KEY}，{key} 按台取不了")
    if text is None:
        raise ByHostValueUnavailable(f"多机配置 {configuration_path} 里没写 {key}（三个按台的键都要写）；{BY_HOST_HOWTO}")
    return by_host_values(text, hosts_text.split(), key, environment)


def emitted_values(text):
    """判法打的 KEY=<printf %q 的值> 一行一个。"""
    values = {}
    for line in text.splitlines():
        if "=" in line:
            key, quoted = line.split("=", 1)
            words = shlex.split(quoted) if quoted else []
            values[key] = words[0] if words else ""
    return values


def configuration_value(path, name):
    """直接读配置里一个键（configured_by_host 与弄坏开关 first-item-taken 用）：以「名字=」起头的最后一行，去一层成对引号；没有这一行交 None。"""
    found = None
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if line.startswith(name + "="):
                found = line.split("=", 1)[1].strip()
    if found and len(found) >= 2 and found[0] == found[-1] and found[0] in "'\"":
        found = found[1:-1]
    return found


def resolve(configuration_path=None, environment=None):
    """主机数组里这一台与第二台（文件头）。取不到抛 AddressesUnavailable。"""
    environment = dict(os.environ if environment is None else environment)
    if configuration_path and broken("first-item-taken", environment):
        items = (configuration_value(configuration_path, "HOSTS") or "").split()
        if items:
            return Addresses("", items[0])
    command = ["bash", CONFIGURATION_CHECK, "--emit-addresses"]
    if configuration_path:
        command += ["--configuration", configuration_path]
    command.append(REPOSITORY_ROOT)
    try:
        completed = subprocess.run(command, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=CHECK_SECONDS, env=environment)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise AddressesUnavailable(f"起不了多机配置判法（{type(error).__name__}: {error}）") from error
    if completed.returncode != 0:
        reason = next((line.strip() for line in completed.stdout.splitlines() if "✗" in line), f"判法退 {completed.returncode}")
        raise AddressesUnavailable(f"多机配置判不过：{reason}")
    values = emitted_values(completed.stdout)
    if not values.get("THIS_HOST_ADDRESS") or not values.get("PEER_HOST_ADDRESS"):
        raise AddressesUnavailable("多机配置判得过，判法却没打 THIS_HOST_ADDRESS 或 PEER_HOST_ADDRESS")
    return Addresses(values["THIS_HOST_ADDRESS"], values["PEER_HOST_ADDRESS"])


def peer_ssh_address(configuration_path, environment=None):
    """第二台的 ssh 地址：→ (地址或 None, 取不到的原因连出路)。用户名归各台自己的 ~/.ssh/config（Host <地址>）。"""
    try:
        return resolve(configuration_path, environment).peer, ""
    except AddressesUnavailable as error:
        return None, f"{error}；{HOWTO}"


# ── 自证 ──
SELFTEST_CONFIGURATION = ("BUILD_ROOT_BY_HOST={build_roots}\nCARGO_BIN_DIRECTORY_BY_HOST={cargo_directories}\nSLICE_MEMORY_CAP_BY_HOST={slice_caps}\n"
                          "QUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES={across}\n"
                          "ENABLE_GPU=0\nNFS_SHARED_ROOTS={root}\nSHARED_DATA_ROOT={root}/data-root\nHOSTS={hosts}\n")


def selftest_by_host(hosts, text_of_address):
    """自证配置里一个按台的键：HOSTS 每一项一个 <地址>=<值>。"""
    return " ".join(f"{address}={text_of_address(address)}" for address in hosts.split())


def selftest():
    failures, cells = [], 0
    with tempfile.TemporaryDirectory(prefix="multi-host-addresses-selftest.") as work:
        def configuration(name, hosts, across=1):
            path = os.path.join(work, name)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(SELFTEST_CONFIGURATION.format(
                    across=across, root=work, hosts=hosts, build_roots=selftest_by_host(hosts, lambda address: f"/selftest/build-root-of-{address}"),
                    cargo_directories=selftest_by_host(hosts, lambda address: f"/selftest/cargo-of-{address}"),
                    slice_caps=selftest_by_host(hosts, lambda address: "2G")))
            return path

        def check(name, good, detail):
            nonlocal cells
            cells += 1
            print(f"  {'✓' if good else '✗'} multi_host_addresses 自证：{name}" + ("" if good else f"：{detail}"))  # gate-lint:detail
            if not good:
                failures.append(name)

        local_environment = dict(os.environ)
        peer_environment = dict(local_environment, HOME="/nonexistent/x", SINGLEFS_MULTI_HOST_CONFIG=os.path.join(work, "absent.env"))
        other_first = configuration("other-first.env", "192.0.2.81 127.0.0.1")
        local_view = peer_ssh_address(other_first, local_environment)
        check("第二台是数组里不是自己的那一项（写在头一项的也认得出）", local_view == ("192.0.2.81", ""), f"取到 {local_view}")
        peer_view = peer_ssh_address(other_first, peer_environment)
        check("第二台的条件（家目录不在、环境指到不在的配置）下取得一样", peer_view == local_view, f"取到 {peer_view}")
        off_view = peer_ssh_address(configuration("off.env", "127.0.0.1 192.0.2.81", across=0), local_environment)
        check("双机关着也照取（判法 --emit-addresses 不看开关）", off_view == ("192.0.2.81", ""), f"取到 {off_view}")
        address, why = peer_ssh_address(configuration("absent-self.env", "192.0.2.82 192.0.2.81"), local_environment)
        check("这一台不在数组里：取不到、点名判法的原因", address is None and "没有一项是这一台网卡上的地址" in why, f"取到 {address}，说的是「{why}」")
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--configuration", other_first, "peer"],
                                   capture_output=True, text=True, env=local_environment)
        check("命令行：peer 打一行第二台的地址、退 0", completed.returncode == 0 and completed.stdout == "192.0.2.81\n",
              f"退 {completed.returncode}，打「{completed.stdout.strip()}」，{completed.stderr.strip()[-200:]}")

        # 按台的键（文件头「按台的三个键」）：各台取各台那一项（写在头一项的不是这一台也照地址取），值里自己带的 = 照留
        def by_host_outcome(text, hosts=("127.0.0.1", "192.0.2.81")):
            try:
                return by_host_values(text, list(hosts), "SLICE_MEMORY_CAP_BY_HOST"), ""
            except ByHostValueUnavailable as error:
                return None, str(error)
        values, why = by_host_outcome("192.0.2.81=/peer/a=b 127.0.0.1=/this/c")
        check("按台取值：各台取各台那一项，值里的 = 照留", values == {"192.0.2.81": "/peer/a=b", "127.0.0.1": "/this/c"} and not why,
              f"取到 {values}，说的是「{why}」")
        configured = configured_by_host(other_first, "BUILD_ROOT_BY_HOST")
        check("按台取值：照配置里的 HOSTS 与键切（configured_by_host）",
              configured == {"192.0.2.81": "/selftest/build-root-of-192.0.2.81", "127.0.0.1": "/selftest/build-root-of-127.0.0.1"}, f"取到 {configured}")
        values, why = by_host_outcome("127.0.0.1=50G")
        check("按台的键少了一台：判写坏、点名缺的那一台", values is None and "少了 192.0.2.81" in why, f"取到 {values}，说的是「{why}」")
        values, why = by_host_outcome("127.0.0.1=50G 192.0.2.81=50G 192.0.2.99=50G")
        check("按台的键多一项不在 HOSTS 里的地址：判写坏、点名那一项", values is None and "192.0.2.99 不在 HOSTS 里" in why, f"取到 {values}，说的是「{why}」")
        values, why = by_host_outcome("127.0.0.1=50G 127.0.0.1=40G 192.0.2.81=50G")
        check("按台的键同一台写两遍：判写坏", values is None and "127.0.0.1 写了两遍" in why, f"取到 {values}，说的是「{why}」")
        values, why = by_host_outcome("127.0.0.1=50G 192.0.2.81")
        check("按台的键有一项没有 =：判写坏", values is None and "没有 =" in why, f"取到 {values}，说的是「{why}」")
    print(f"multi_host_addresses 自证：{cells - len(failures)} 格对、{len(failures)} 格错（共 {cells} 格）")
    return 1 if failures else 0


def main(arguments):
    if os.environ.get(BREAK_VARIABLE):
        unknown = [item for item in os.environ[BREAK_VARIABLE].split(",") if item not in KNOWN_BREAKS]
        if unknown or not (arguments == ["--selftest"] or os.environ.get(SELFTEST_VARIABLE) == "1"):
            print(f"  ✗ {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 只在 --selftest 里认（认的项：{'、'.join(KNOWN_BREAKS)}），这一次什么都没取", file=sys.stderr)
            print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/hosts/multi_host_addresses.py --selftest", file=sys.stderr)
            return 2
    if arguments == ["--selftest"]:
        os.environ[SELFTEST_VARIABLE] = "1"
        return selftest()
    configuration_path = None
    if arguments[:1] == ["--configuration"] and len(arguments) >= 2:
        configuration_path, arguments = arguments[1], arguments[2:]
    if arguments not in (["peer"], ["this"]):
        print(f"  ✗ 认不出的用法：{' '.join(arguments)}", file=sys.stderr)
        print("     → 怎么办：python3 scripts/hosts/multi_host_addresses.py [--configuration <配置文件>] peer|this，或 --selftest", file=sys.stderr)
        return 2
    try:
        addresses = resolve(configuration_path)
    except AddressesUnavailable as error:
        print(f"  ✗ 取不到主机数组里的地址：{error}", file=sys.stderr)
        print(f"     → 怎么办：{HOWTO}", file=sys.stderr)
        return 1
    print(addresses.peer if arguments == ["peer"] else addresses.this)
    return 0


if __name__ == "__main__":
    sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
