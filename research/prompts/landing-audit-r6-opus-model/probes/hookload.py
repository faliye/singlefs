"""把钩子（bash 文件里 python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY' … PY 那一段）里的 python 当模块载进来，不跑 main()。
load(<钩子文件>, <钩子目录>) → 模块；钩子目录交给它当 sys.argv[1]（它从那里 import 同目录的库）。"""
import os, sys, types

def embedded(path):
    lines = open(path, encoding="utf-8").read().split("\n")
    start = next(index for index, line in enumerate(lines) if line.startswith('python3 /dev/fd/3 "$HOOK_DIR"') and line.endswith("3<<'PY'"))
    end = next(index for index in range(start + 1, len(lines)) if lines[index] == "PY")
    body = lines[start + 1:end]
    if body and body[-1].startswith("sys.exit(main("):
        body = body[:-1]
    return "\n".join(body)

def load(path, hook_dir, name):
    saved_argv, saved_path = list(sys.argv), list(sys.path)
    for cached in ("lib_work_root", "lib_three_way_legs", "lib_prompt_scratch_line", "lib_writer_lock", "lib_heavy_tests", "lib_shell_words", "lib_write_scope"):
        sys.modules.pop(cached, None)
    sys.argv = [path, hook_dir]
    module = types.ModuleType(name)
    module.__file__ = path
    try:
        exec(compile(embedded(path), path, "exec"), module.__dict__)
    finally:
        sys.argv = saved_argv
        sys.path[:] = saved_path
    return module
