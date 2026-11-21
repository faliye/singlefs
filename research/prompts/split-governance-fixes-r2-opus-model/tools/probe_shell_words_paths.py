#!/usr/bin/env python3
"""探针：lib_shell_words.py 按目录各存一份部分模块，同一个进程里按几种路径写法载入，用的是不是同一份。
用法：probe_shell_words_paths.py --frozen <冻结副本> --work <输出目录>
在 <输出目录>/shell-words/ 下建：
  A/       冻结副本的 lib_shell_words.py、lib_shell_words_core.py、lib_shell_words_embedded.py，入口与 core 末尾各加一行标记 "A"；A/inner/ 是空目录
  W/       同三份，标记 "W"；W/link 是指向 A/inner 的符号链接
  L        指向 A 的符号链接
同一个进程里按下面几种写法各载入一次入口（spec_from_file_location，模块名各不相同），每次一行：
  PROBE_SHELL_WORDS form=<写法> entry_mark=<真执行的入口> core_mark=<它用的 core> library_directory=<它认的目录> same_core_as_absolute=<是否与 A 的绝对路径那一次同一份 core> shellword_same_class=<ShellWord 是否同一个类>
写法：absolute（A 的绝对路径）、relative（chdir 到 A 之后写 lib_shell_words.py）、directory-symlink（L/lib_shell_words.py）、
     dot-dot（A/inner/../lib_shell_words.py）、symlink-then-dot-dot（W/link/../lib_shell_words.py：内核先解链接到 A/inner、再上一层到 A；按字面去掉 .. 是 W）、
     other-directory（W 的绝对路径）。
"""
import argparse, importlib.util, os, shutil, sys

NAMES = ("lib_shell_words.py", "lib_shell_words_core.py", "lib_shell_words_embedded.py")


def build(frozen, base):
    for mark in ("A", "W"):
        directory = os.path.join(base, mark)
        os.makedirs(directory)
        for name in NAMES:
            shutil.copy(os.path.join(frozen, ".claude", "hooks", name), os.path.join(directory, name))
        for name, variable in (("lib_shell_words.py", "PROBE_ENTRY_MARK"), ("lib_shell_words_core.py", "PROBE_CORE_MARK")):
            with open(os.path.join(directory, name), "a", encoding="utf-8") as handle:
                handle.write(f'\n{variable} = "{mark}"\n')
    os.makedirs(os.path.join(base, "A", "inner"))
    os.symlink(os.path.join(base, "A", "inner"), os.path.join(base, "W", "link"))
    os.symlink(os.path.join(base, "A"), os.path.join(base, "L"))


def load(path, module_name):
    specification = importlib.util.spec_from_file_location(module_name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def core_of(module):
    return sys.modules.get(module._part_module_key("lib_shell_words_core"))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    base = os.path.join(os.path.realpath(arguments.work), "shell-words")
    build(os.path.realpath(arguments.frozen), base)
    sys.dont_write_bytecode = True
    forms = [("absolute", os.path.join(base, "A", "lib_shell_words.py"), None),
             ("relative", "lib_shell_words.py", os.path.join(base, "A")),
             ("directory-symlink", os.path.join(base, "L", "lib_shell_words.py"), None),
             ("dot-dot", os.path.join(base, "A", "inner") + "/../lib_shell_words.py", None),
             ("symlink-then-dot-dot", os.path.join(base, "W", "link") + "/../lib_shell_words.py", None),
             ("other-directory", os.path.join(base, "W", "lib_shell_words.py"), None)]
    reference = None
    start = os.getcwd()
    for index, (form, path, directory) in enumerate(forms):
        os.chdir(directory or start)
        try:
            module = load(path, f"probe_shell_words_{index}")
        except Exception as error:   # 载入失败也是读数：照打一行
            print(f"PROBE_SHELL_WORDS form={form} load_error={type(error).__name__}: {str(error)[:200]}", flush=True)
            continue
        finally:
            os.chdir(start)
        core = core_of(module)
        if reference is None:
            reference = (core, module.ShellWord)
        print(f"PROBE_SHELL_WORDS form={form} entry_mark={getattr(module, 'PROBE_ENTRY_MARK', '?')} core_mark={getattr(core, 'PROBE_CORE_MARK', '?')} "
              f"library_directory={module._LIBRARY_DIRECTORY} same_core_as_absolute={core is reference[0]} "
              f"shellword_same_class={module.ShellWord is reference[1]}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
