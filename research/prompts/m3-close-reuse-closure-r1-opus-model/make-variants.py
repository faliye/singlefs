"""从一棵树（被判的那份脚本在里面）造两份变体树，各只换一份文件（其余硬链到原树）：
  edge-cut       run-with-memory-cap.sh 里 HostLookup.query 起 host-resources.py 的那一行换成起一份字面的内联 python（"pass"）：
                 等于按「调用边」把 HostLookup → host-resources.py 那一条边从闭包里去掉，别的边不动
  hostlookup-text  HostLookup.run 打的「没查到别的机器」那一句改几个字（看它落在哪个单元的指纹里）
  broken-import  .claude/hooks/lib_long_run.py 在 import lib_heavy_tests 之后加一行模块级 raise（import 时就失败）
用法：python3 make-variants.py <原树> <放变体的目录>"""
import os, shutil, subprocess, sys

source, target = sys.argv[1], sys.argv[2]
os.makedirs(target, exist_ok=True)
EDITS = {
    "edge-cut": ("scripts/hosts/run-with-memory-cap.sh",
                 'process = self.process = subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,',
                 'process = self.process = subprocess.Popen([sys.executable, "-c", "pass"], stdout=output, stderr=errors,'),
    "hostlookup-text": ("scripts/hosts/run-with-memory-cap.sh", 'say(f"run-with-memory-cap: 没查到别的机器：{problem}")',
                        'say(f"run-with-memory-cap: 没查到别的机器（只改了这几个字）：{problem}")'),
    "broken-import": (".claude/hooks/lib_long_run.py", "import lib_heavy_tests  # noqa: E402\n",
                      'import lib_heavy_tests  # noqa: E402\nraise RuntimeError("attack: lib_long_run import-time failure")\n'),
}
for name, (relative, old, new) in EDITS.items():
    tree = os.path.join(target, name)
    if os.path.exists(tree):
        shutil.rmtree(tree)
    subprocess.run(["cp", "-al", source, tree], check=True)
    path = os.path.join(tree, relative)
    text = open(path, encoding="utf-8").read()
    if text.count(old) != 1:
        sys.exit(f"{name}: {relative} 里要换的那一句命中 {text.count(old)} 次（要恰好 1 次）")
    temporary = path + ".variant"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
    os.chmod(temporary, os.stat(path).st_mode)
    os.replace(temporary, path)
    print(f"variant={name} file={relative}")
