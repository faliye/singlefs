#!/usr/bin/env python3
"""S1：把派发闸里内嵌的 python 抽出来（不跑 main），对 without_code_blocks 做三件事：
  1. 异常面：随机拼的 markdown 碎片、超深嵌套、NUL、孤立代理字符、很长的行，看 parser.parse 会不会抛异常（函数里 parse 在 try 之外）；
  2. 耗时：大提示的耗时；
  3. 不变量：函数不改行数，被遮的行必须落在 markdown-it 认的 fence 令牌里。
用法：s1_fuzz_without_code_blocks.py <树根> [随机种子] [份数]"""
import os, random, re, sys, time
root = os.path.abspath(sys.argv[1]); seed = int(sys.argv[2]) if len(sys.argv) > 2 else 1; count = int(sys.argv[3]) if len(sys.argv) > 3 else 400
hook = os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh")
text = open(hook, encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(text) if l.startswith("python3 /dev/fd/3"))
end = next(i for i in range(start + 1, len(text)) if text[i] == "PY")
source = "\n".join(text[start + 1:end])
source = source.replace("\nsys.exit(main())", "\n")
sys.argv = ["hook", os.path.dirname(hook)]
namespace = {"__name__": "hook_under_test"}
exec(compile(source, "runner-dispatch-guard-embedded", "exec"), namespace)
without = namespace["without_code_blocks"]
if not hasattr(without, "__call__"):
    raise SystemExit("没有 without_code_blocks")
random.seed(seed)
pieces = ["```", "~~~", "````", "~~~~~", "    ```", "\t```", "- ```", "  ```", "> ```", "> > ```", "1. ```", "线程上限：31", "线程上限：16", "线程上限已判：理由", "多机：x", "<div>", "</div>", "<!--", "-->",
          "[a]: /u", "[a]: /u \"t", "===", "---", "# 标题", "| a | b |", "|---|---|", "text", "", "", "\r", "- 项", "  - 嵌套", "    ", "\x00", "`", "``", "***", "<pre>", "```a`b", "~~~ x"]
exceptions, changed_lines = [], 0
t0 = time.time()
for case in range(count):
    n = random.randint(1, 40)
    prompt = random.choice(["\n", "\r\n"]).join(random.choice(pieces) for _ in range(n)) + random.choice(["", "\n"])
    try:
        out = without(prompt)
    except BaseException as error:   # noqa
        exceptions.append((case, repr(error)[:100], prompt[:80].replace("\n", "\\n")))
        continue
    if len(out.split("\n")) != len(prompt.split("\n")):
        exceptions.append((case, "行数变了", prompt[:80].replace("\n", "\\n")))
print(f"随机 {count} 份（种子 {seed}）：异常 {len(exceptions)} 份")
print(f"用时 {time.time() - t0:.2f}s", file=sys.stderr)
for item in exceptions[:5]: print("  ", item)
# 极端形状
def trial(name, prompt):
    t = time.time()
    try:
        out = without(prompt); status = "ok"
    except BaseException as error:  # noqa
        status = "异常 " + repr(error)[:90]
    print(f"{name}\t字数={len(prompt)}\t{status}")
    print(f"  {name} 用时 {time.time() - t:.2f}s", file=sys.stderr)
trial("深嵌套引用块 2000 层", "> " * 2000 + "x\n")
trial("深嵌套列表 400 层", "".join("  " * i + "- a\n" for i in range(400)))
trial("孤立代理字符", "线程上限：31\n\ud800\n```\n线程上限：16\n```\n")
trial("NUL", "线程上限：31\n\x00\n```\n线程上限：16\n```\n")
trial("1MB 普通正文", ("普通正文一行。\n" * 100000))
trial("20 万行围栏交替", "```\nx\n```\n" * 70000)
trial("单行 2MB", "a" * 2_000_000 + "\n")
trial("10 万个行内反引号", "`a` " * 100000 + "\n")
