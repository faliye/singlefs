"""先验后换自检（入口 hook-swap.py 的 --selftest）的一块：样本钩子、样本调用方与喂法。
"""

import os

from hook_swap_base import HOOKS_DIRECTORY

SELFTEST_GUARD = """#!/usr/bin/env bash
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 /dev/fd/3 "$HOOK_DIR" 3<<'PY'
import importlib.util, json, os, sys
spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(sys.argv[1], "fake_lib.py"))
lib = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lib)
command = json.load(sys.stdin).get("tool_input", {}).get("command", "")
if lib.refused(command):
    print("✗ 被拒：" + command, file=sys.stderr)
    print("     → 怎么办：自证用的假钩子，只按假库的判法答，没有别的出路", file=sys.stderr)
    sys.exit(2)
sys.exit(0)
PY
"""
# 按落点判的假钩子（写闸的样子）：file_path 解开符号链接之后落在 __WORK_ROOT__ 里一律放行，落在外面时 .md 拒、别的放行。自证把它写进仓里时 __WORK_ROOT__ 换成草稿目录，
# 所以 .md 的目标喂主工作区拒、喂草稿树（在草稿目录下）放行，别的后缀的目标两处都放行
SELFTEST_SCOPE_GUARD = """#!/usr/bin/env bash
set -uo pipefail
# 版本 1
python3 /dev/fd/3 3<<'PY'
import json, os, sys
work_root = __WORK_ROOT__
landing = os.path.realpath(json.load(sys.stdin).get("tool_input", {}).get("file_path", ""))
inside_work_root = landing.startswith(work_root + os.sep)
if landing.endswith(".md") and not inside_work_root:
    print("✗ 被拒：" + landing + " 是说明文件，在工作根之外", file=sys.stderr)
    print("     → 怎么办：自证用的假钩子，.md 只许写在工作根里，没有别的出路", file=sys.stderr)
    sys.exit(2)
sys.exit(0)
PY
"""
SELFTEST_LIBRARY = ('MARKER = "HEAVY"\n\n\ndef refused(command):\n    return MARKER in command\n\n\n'
                    'def refused_v2(command):\n    return MARKER in command or "*" in command\n')
SELFTEST_LIBRARY_WITHOUT_V2 = 'MARKER = "HEAVY"\n\n\ndef refused(command):\n    return MARKER in command\n'
SELFTEST_FEEDS = [{"label": "只读放行", "command": "ls -la", "exit": 0, "neutral": True},
                  {"label": "重的照拒", "command": "run HEAVY", "exit": 2, "neutral": True},
                  {"label": "新判法：通配照拒", "command": "run GLOB*", "exit": 2}]
SELFTEST_GOOD_EDIT = {"file": ".claude/hooks/fake_lib.py", "old": "    return MARKER in command\n",
                      "new": "    return MARKER in command or \"*\" in command  # 新判法\n"}
SELFTEST_OTHER_SESSION_LINE = "# 别的会话这段时间里加的一行\n"
# 草稿目录占用的样本：被喂到时先写就绪文件、等放行文件出现才退（最多 SELFTEST_WAITING_GUARD_SECONDS 秒）的假钩子，让一趟真的 hook-swap.py 停在「占着草稿目录」那一步
SELFTEST_WAITING_GUARD_SECONDS = 20
SELFTEST_WAITING_GUARD = f"""#!/usr/bin/env bash
set -uo pipefail
# 版本 1
touch "__READY__"
for _ in $(seq 1 {SELFTEST_WAITING_GUARD_SECONDS * 20}); do
    [ -e "__RELEASE__" ] && break
    sleep 0.05
done
exit 0
"""
SELFTEST_WAITING_FEEDS = [{"label": "只读放行", "command": "ls -la", "exit": 0}]
# 摘旧记录那一趟的样本：载入被测的 hook-swap.py，（给了草稿目录就先占住它，）摘 argv 里那份记录的旧记录，摘到 .taking 已放成的当中写标记文件、睡着等被停。
# 参数：被测脚本、记录路径、repository 或 scratch、标记文件、弄坏开关、[先占的草稿目录]
SELFTEST_CLAIM_CHILD = '''import importlib.util
import sys
import time

spec = importlib.util.spec_from_file_location("hook_swap_claim_child", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
record_path, kind, marker, mode = sys.argv[2:6]
if len(sys.argv) > 6:
    held, busy = module.hold_scratch(sys.argv[6], mode, heartbeat_seconds=3600)
    if held is None:
        sys.exit(3)
repository = kind == "repository"
wanted = module.ScratchLock(record_path, module.scratch_lock_content("claim-child-scratch" if repository else None), mode)


def pause():
    with open(marker, "w", encoding="utf-8") as handle:
        handle.write("claiming")
    time.sleep(600)


columns = module.REPOSITORY_LOCK_COLUMNS if repository else module.SCRATCH_LOCK_COLUMNS
module.acquire_record(wanted, columns, 3600, after_taken=pause)
'''
# 脚本规格的样本：scripts/tool.py 退 0 就是验过，scripts/sub/tool.sh 是更深一层的 .sh
SELFTEST_SCRIPT = '#!/usr/bin/env python3\nVERSION = 1\n\n\ndef main():\n    return 0\n\n\nif __name__ == "__main__":\n    raise SystemExit(main())\n'
SELFTEST_SCRIPT_SHELL = "#!/usr/bin/env bash\necho tool\n"
SELFTEST_SCRIPT_VERIFY = "python3 scripts/tool.py"
SELFTEST_SHELL_WITH_VERSION = "#!/usr/bin/env bash\nVERSION=1\necho sample\n"
SELFTEST_PYTHON_VERSION_EDIT = ("VERSION = 1\n", "VERSION = 2\n")
SELFTEST_SHELL_VERSION_EDIT = ("VERSION=1\n", "VERSION=2\n")
# 脚本规格目录的样本：scripts/ 之外的五处每处一份已有的文件（相对仓根 → (内容, 旧串, 新串, 文件权限)），服务的控制脚本照无后缀、带执行位写
SELFTEST_SPEC_DIRECTORY_FILES = {
    ".claude/gate.d/sample-stage.sh": (SELFTEST_SHELL_WITH_VERSION, *SELFTEST_SHELL_VERSION_EDIT, 0o755),
    ".claude/scripts/sample_tool.py": (SELFTEST_SCRIPT, *SELFTEST_PYTHON_VERSION_EDIT, 0o644),
    "task-server/sample_lib.py": (SELFTEST_SCRIPT, *SELFTEST_PYTHON_VERSION_EDIT, 0o644),
    "prune-server/control": (SELFTEST_SHELL_WITH_VERSION, *SELFTEST_SHELL_VERSION_EDIT, 0o755),
    "gpu-verification-server/control": (SELFTEST_SHELL_WITH_VERSION, *SELFTEST_SHELL_VERSION_EDIT, 0o755),
}
# 脚本规格不收的子目录里各一份已有的文件：相对仓根 → (内容, 旧串, 新串, 文件权限, 那一格的名字)
SELFTEST_EXCLUDED_FILES = {
    ".claude/gate.d/fixtures/sample-red/stage.sh": (SELFTEST_SHELL_WITH_VERSION, *SELFTEST_SHELL_VERSION_EDIT, 0o755, "脚本规格：.claude/gate.d/fixtures/ 下的样本不收"),
    "prune-server/target/build-tool.py": (SELFTEST_SCRIPT, *SELFTEST_PYTHON_VERSION_EDIT, 0o755, "脚本规格：prune-server/target/ 下的文件不收"),
    "gpu-verification-server/target/release/run.sh": (SELFTEST_SHELL_WITH_VERSION, *SELFTEST_SHELL_VERSION_EDIT, 0o755, "脚本规格：gpu-verification-server/target/ 下的文件不收"),
    "prune-server/src/main.rs": ("// VERSION=1\nfn main() {}\n", *SELFTEST_SHELL_VERSION_EDIT, 0o644, "脚本规格：prune-server/src/ 下的文件不收"),
    "gpu-verification-server/src/build.py": (SELFTEST_SCRIPT, *SELFTEST_PYTHON_VERSION_EDIT, 0o644, "脚本规格：gpu-verification-server/src/ 下的文件不收"),
}
# 调用方引到的名字的样本（都引 fake_lib.refused_v2）：每份一种载入写法，fake_outer.py 把库交给另一份库 fake_inner.py 里的函数，名字在 fake_inner.py 里引
SELFTEST_CALLER_SAMPLES = {
    "fake_spec_user.py": (
        "import importlib.util\nimport os\n\n\ndef load_library(directory):\n"
        '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        "    library = importlib.util.module_from_spec(spec)\n    spec.loader.exec_module(library)\n    return library\n\n\n"
        "LIBRARY = load_library(os.path.dirname(__file__))\n\n\ndef side_check(command):\n    return LIBRARY.refused_v2(command)\n"),
    "fake-side-guard.sh": (
        '#!/usr/bin/env bash\nHOOK_DIR="$(dirname "$0")"\npython3 /dev/fd/3 "$HOOK_DIR" 3<<\'PY\'\nimport os, sys\nsys.path.insert(0, sys.argv[1])\n'
        "import fake_lib as side_library\nprint(side_library.refused_v2(sys.argv[2]))\nPY\n"),
    "fake_holder.py": (
        "import os\n\n\nclass Holder:\n    def __init__(self, loader, directory):\n"
        '        self.library = loader.load_module("fake_lib", os.path.join(directory, "fake_lib.py"))\n\n'
        "    def check(self, command):\n        return self.library.refused_v2(command)\n"),
    "fake_param.py": (
        "import importlib.util\nimport os\n\n\ndef run(library, command):\n    return library.refused_v2(command)\n\n\ndef main(directory):\n"
        '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        '    module = importlib.util.module_from_spec(spec)\n    return run(module, "x")\n'),
    "fake_container.py": (
        "import importlib.util\nimport os\n\nCACHE = []\n\n\ndef library():\n    if not CACHE:\n"
        '        spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(os.path.dirname(__file__), "fake_lib.py"))\n'
        "        module = importlib.util.module_from_spec(spec)\n        CACHE.append(module)\n    return CACHE[0]\n\n\n"
        "def check(command):\n    return library().refused_v2(command)\n"),
    "fake_global.py": (
        "import importlib.util\nimport os\n\nfake_lib = None\n\n\ndef load(directory):\n    global fake_lib\n"
        '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        "    module = importlib.util.module_from_spec(spec)\n    fake_lib, marker = module, module.MARKER\n    return marker\n\n\n"
        "def use(command):\n    return fake_lib.refused_v2(command)\n"),
    "fake_inner.py": "def helper(library, command):\n    return library.refused_v2(command)\n",
    "fake_outer.py": (
        "import importlib.util\nimport os\n\n\ndef run(directory):\n"
        '    library_spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        "    library = importlib.util.module_from_spec(library_spec)\n"
        '    inner_spec = importlib.util.spec_from_file_location("fake_inner", os.path.join(directory, "fake_inner.py"))\n'
        '    inner = importlib.util.module_from_spec(inner_spec)\n    return inner.helper(library, "x")\n'),
}
SELFTEST_CALLERS_OF_REFUSED_V2 = tuple(name for name in SELFTEST_CALLER_SAMPLES if name != "fake_outer.py")
# 载入的库交给认不出的调用（register 不在本文件、也不是另一份目标库的函数）：去向跟不上，要列成本次未判
SELFTEST_OPAQUE_CALLER = ("import importlib.util\nimport os\n\nfrom elsewhere import register\n\n\ndef load(directory):\n"
                          '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
                          "    module = importlib.util.module_from_spec(spec)\n    register(module)\n")


# 按字符串键存进 sys.modules 再取出（container_entry）：同一份调用方按路径载入三份库、各写进 sys.modules，取出时按键分开认——
# library() 取 fake_lib 存进去时的名字（refused_v2 归 fake_lib）；client() 取不跟着走的 registry_client 载入时的名字（client_only 不核）；
# peer() 取钩子目录里另一份库 fake_peer 的名字（peer_only 归 fake_peer）；lookup() 的键是形参，折不出，列成本次未判
SELFTEST_REGISTRY_CALLER = """import importlib.util
import os
import sys

DIRECTORY = os.path.dirname(__file__)


def library():
    specification = importlib.util.spec_from_file_location('fake_lib_in_registry', os.path.join(DIRECTORY, 'fake_lib.py'))
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    return sys.modules['fake_lib_in_registry']


def client():
    name = 'registry_client_in_registry'
    if name not in sys.modules:
        specification = importlib.util.spec_from_file_location(name, os.path.join(DIRECTORY, '..', 'registry', 'registry_client.py'))
        module = importlib.util.module_from_spec(specification)
        sys.modules[name] = module
        specification.loader.exec_module(module)
    return sys.modules[name]


def peer():
    name = 'fake_peer_in_registry'
    specification = importlib.util.spec_from_file_location(name, os.path.join(DIRECTORY, 'fake_peer.py'))
    sys.modules[name] = importlib.util.module_from_spec(specification)
    return sys.modules[name]


def check(command):
    return library().refused_v2(command) and client().client_only(command) and peer().peer_only(command)


def lookup(key, command):
    return sys.modules[key].refused_v2(command)
"""
SELFTEST_PEER_LIBRARY = """def peer_only(command):
    return command
"""
# 本文件没按键存过 sys.modules（registry_entry）与只用 from 库 import 名字引库（names_library_without_binding）的调用方，都引 fake_lib.refused_v2：
# fake_registry_read.py 先 import 成 loaded 再往 sys.modules 按库名取，refused_v2 只在按库名取的那一处引；fake_from_import.py 的 refused_v2 是 from 那一行引来的；
# fake_registry_unfolded.py 的 sys.modules 键是形参，折不出
SELFTEST_REGISTRY_READ_CALLER = """import sys
import fake_lib as loaded


def check(command):
    return loaded.MARKER, sys.modules['fake_lib'].refused_v2(command)
"""
SELFTEST_FROM_IMPORT_CALLER = """from fake_lib import refused_v2


def use(command):
    return refused_v2(command)
"""
SELFTEST_REGISTRY_UNFOLDED_CALLER = """import sys
import fake_lib as loaded


def lookup(key):
    return loaded.MARKER, sys.modules[key]
"""
# sys.modules 的另两种取法（都引 fake_lib.refused_v2，在第 6 行）：.get(键, 缺省)；from sys import modules 之后的 modules[键]。fake_registry_get_unfolded.py 的 .get 键是形参，折不出
SELFTEST_REGISTRY_GET_CALLER = """import sys
import fake_lib as loaded


def check(command):
    return loaded.MARKER, sys.modules.get('fake_lib', None).refused_v2(command)
"""
SELFTEST_REGISTRY_ALIAS_CALLER = """from sys import modules
import fake_lib as loaded


def check(command):
    return loaded.MARKER, modules['fake_lib'].refused_v2(command)
"""
SELFTEST_REGISTRY_GET_UNFOLDED_CALLER = """import sys
import fake_lib as loaded


def lookup(key):
    return loaded.MARKER, sys.modules.get(key)
"""
# 同名的库：.claude/hooks/fake_lib.py 是要核的那一份；scripts/ 下另有一份同名的 fake_lib.py（里面有 refused_v2）和几份各自解到不同那一份的调用方，都引 fake_lib.refused_v2。
# 调用方在 scripts/<目录>/ 下，目录名说它怎么找库：same_directory 自己目录里就有一份同名的；inserted_* 往 sys.path 最前放目录（目录从 __file__ 折出）；
# loaded_* 按路径载入；unfolded_path 往 sys.path 放的目录取自命令行参数，折不出
SELFTEST_SHADOW_LIBRARY = "def refused_v2(command):\n    return command\n"
SELFTEST_SHADOW_DIRECTORY = "scripts/shadow_library"
SELFTEST_SAME_DIRECTORY_CALLER = "from fake_lib import refused_v2\n\n\ndef use(command):\n    return refused_v2(command)\n"
SELFTEST_FAR_CALLER = "import fake_lib\n\n\ndef use(command):\n    return fake_lib.refused_v2(command)\n"


def selftest_inserted_caller(target):
    """往 sys.path 最前放 <仓根>/<target> 再 import fake_lib 的调用方（引 refused_v2 在第 9 行）。"""
    parts = ", ".join(f'"{part}"' for part in target.split("/"))
    return ("import os\nimport sys\n\n"
            f'sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", {parts}))\n'
            "import fake_lib\n\n\ndef use(command):\n    return fake_lib.refused_v2(command)\n")


def selftest_loaded_caller(target):
    """按路径载入 <仓根>/<target>/fake_lib.py 的调用方（引 refused_v2 在第 10 行）。"""
    parts = ", ".join(f'"{part}"' for part in target.split("/"))
    return ("import importlib.util\nimport os\n\n"
            f'LIBRARY_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", {parts}, "fake_lib.py")\n'
            'SPECIFICATION = importlib.util.spec_from_file_location("fake_lib", LIBRARY_PATH)\n'
            "LIBRARY = importlib.util.module_from_spec(SPECIFICATION)\n\n\n"
            "def use(command):\n    return LIBRARY.refused_v2(command)\n")


SELFTEST_UNFOLDED_PATH_CALLER = "import sys\n\nsys.path.insert(0, sys.argv[1])\nimport fake_lib\n\n\ndef use(command):\n    return fake_lib.refused_v2(command)\n"
# 起头那一行以反斜杠收尾的 heredoc（python3 - … <<'PY' 后面接 || exit 1）：正文从续行结束的下一行起，第 2 行是起头、第 3 行是续行、第 4 行起是正文
SELFTEST_CONTINUED_HEREDOC = "#!/usr/bin/env bash\npython3 - \"$1\" <<'PY' \\\n    || exit 1\nimport sys\nprint(sys.argv)\nPY\n"
SELFTEST_CONTINUED_HEREDOC_BROKEN = SELFTEST_CONTINUED_HEREDOC.replace("print(sys.argv)\n", "print(sys.argv\n")    # 正文里括号没收
# 同一形里引 fake_lib.refused_v2（在文件第 8 行：起头第 3 行、续行第 4 行，正文第 4 行是引的那一句）
SELFTEST_CONTINUED_HEREDOC_CALLER = ('#!/usr/bin/env bash\nHOOK_DIR="$(dirname "$0")"\npython3 /dev/fd/3 "$HOOK_DIR" 3<<\'PY\' \\\n    || exit 1\n'
                                     "import os, sys\nsys.path.insert(0, sys.argv[1])\nimport fake_lib\nprint(fake_lib.refused_v2(sys.argv[2]))\nPY\n")
# 调用方 import 的别的模块替它往 sys.path 放目录（scripts/<目录>/ 下一份放目录的模块加一份调用方，调用方都在第 6 行引 fake_lib.refused_v2；
# 仓里叫 fake_lib.py 的不止要核的那一份）：path_setter 往最前放 .claude/hooks/（调用方自己所在的目录里另有一份同名的 fake_lib.py 挡在后面）、
#   argument_setter 放的目录取自命令行参数（折不出）、environment_setter 放的目录取自 os.environ.get(…) or 缺省（在 sys.path 后面放），
#   另有调用方自己放的目录取自 import 进来的模块的常量（attribute_side.HOOKS，引在第 10 行）
SELFTEST_PATH_SETTER = ("import os\nimport sys\n\n"
                        'sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "hooks"))\n')
SELFTEST_ARGUMENT_SETTER = "import sys\n\nsys.path.insert(0, sys.argv[1])\n"
SELFTEST_ENVIRONMENT_SETTER = ("import os\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n"
                               'ROOT = os.environ.get("FAKE_ROOT_FOR_A_COPY") or os.path.abspath(os.path.join(HERE, "..", ".."))\n'
                               'HOOKS = os.path.join(ROOT, ".claude", "hooks")\nif HOOKS not in sys.path:\n    sys.path.insert(1, HOOKS)\n')
SELFTEST_ATTRIBUTE_SIDE = 'import os\n\nHERE = os.path.dirname(os.path.abspath(__file__))\nHOOKS = os.path.join(HERE, "..", "..", ".claude", "hooks")\n'
SELFTEST_ATTRIBUTE_CALLER = ("import sys\n\nimport attribute_side\n\nsys.path.insert(1, attribute_side.HOOKS)\nimport fake_lib\n\n\n"
                             "def use(command):\n    return fake_lib.refused_v2(command)\n")


def selftest_setter_caller(setter):
    """先 import 往 sys.path 放目录的模块 setter、再 import fake_lib 的调用方（引 refused_v2 在第 6 行）。"""
    return f"import {setter}  # noqa: F401\nimport fake_lib\n\n\ndef use(command):\n    return fake_lib.refused_v2(command)\n"


# 往 sys.path 放目录的写法在函数里：名字只在那个函数里赋值、另一个函数里另有同名的名字（整份源码里赋了两处），放 <仓根>/.claude/hooks
SELFTEST_LOCAL_SETTER = ("import os\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
                         "def add_hooks_directory():\n"
                         '    directory = os.path.join(HERE, "..", "..", ".claude", "hooks")\n'
                         "    if directory not in sys.path:\n        sys.path.insert(0, directory)\n\n\n"
                         'def other_function():\n    directory = "somewhere"\n    return directory\n')


def selftest_tail_setter(tail):
    """往 sys.path 最前放 <形参>/<tail>（前半截折不出，末尾几段写死）的模块。"""
    parts = ", ".join(f'"{part}"' for part in tail.split("/"))
    return f"import os\nimport sys\n\n\ndef add_directory(repo_root):\n    directory = os.path.join(repo_root, {parts})\n    sys.path.insert(0, directory)\n"


def selftest_function_setter(exact):
    """往 sys.path 最前放不带参数的函数 shared_directory() 返回的目录的模块：函数里 directory 赋了两处，一处折得出 <仓根>/<exact>、一处前半截折不出（<命令行参数>/scripts/shared_side），
    另有 return None。"""
    parts = ", ".join(f'"{part}"' for part in exact.split("/"))
    return ("import os\nimport sys\n\nLOADER_DIRECTORY = os.path.dirname(os.path.realpath(__file__))\n\n\n"
            "def shared_directory():\n"
            f'    directory = os.path.join(LOADER_DIRECTORY, "..", "..", {parts})\n'
            "    if os.path.isdir(directory):\n        return os.path.normpath(directory)\n"
            '    directory = os.path.join(os.path.dirname(sys.argv[0]), "scripts", "shared_side")\n'
            "    if os.path.isdir(directory):\n        return os.path.normpath(directory)\n    return None\n\n\n"
            "def add_shared_directory():\n    directory = shared_directory()\n"
            "    if directory is not None and directory not in sys.path:\n        sys.path.insert(0, directory)\n")


# 载入与绑定都没认出而文本里点名了 fake_lib 的写法：不带 as 的带点 import、SourceFileLoader、runpy.run_path
SELFTEST_UNRECOGNIZED_CALLERS = {
    "scripts/unrecognized/dotted_import_caller.py": "import scripts.shadow_library.fake_lib\n\n\ndef use():\n    return scripts.shadow_library.fake_lib.refused_v2()\n",
    "scripts/unrecognized/source_file_loader_caller.py": (
        "import importlib.machinery\nimport os\n\n"
        'LIBRARY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".claude", "hooks", "fake_lib.py")\n\n\n'
        'def use():\n    return importlib.machinery.SourceFileLoader("fake_lib", LIBRARY).load_module().refused_v2()\n'),
    "scripts/unrecognized/run_path_caller.py": (
        "import os\nimport runpy\n\n"
        'LIBRARY = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".claude", "hooks", "fake_lib.py")\n\n\n'
        'def use():\n    return runpy.run_path(LIBRARY)["refused_v2"]()\n'),
}
# X.__module__ 当 sys.modules 的键，X 是 from fake_lib import X 引进来的（引 fake_lib.refused_v2 在第 8 行）
SELFTEST_MODULE_KEY_CALLER = ("import sys\n\nfrom fake_lib import refused\n\n\ndef use(command):\n    module = sys.modules[refused.__module__]\n"
                              "    return refused(command), module.refused_v2(command)\n")
# 同名的库的调用方各放在哪：相对仓根的路径 → (内容, 解到的是 fake_lib 的哪一份：True 是 .claude/hooks/ 下要核的那份)
SELFTEST_SHADOW_CALLERS = {
    f"{SELFTEST_SHADOW_DIRECTORY}/same_directory_caller.py": (SELFTEST_SAME_DIRECTORY_CALLER, False),
    "scripts/inserted_to_shadow/inserted_to_shadow_caller.py": (selftest_inserted_caller(SELFTEST_SHADOW_DIRECTORY), False),
    "scripts/inserted_to_hooks/inserted_to_hooks_caller.py": (selftest_inserted_caller(HOOKS_DIRECTORY.replace(os.sep, "/")), True),
    "scripts/loaded_shadow/loaded_shadow_caller.py": (selftest_loaded_caller(SELFTEST_SHADOW_DIRECTORY), False),
    "scripts/loaded_hooks/loaded_hooks_caller.py": (selftest_loaded_caller(HOOKS_DIRECTORY.replace(os.sep, "/")), True),
}
