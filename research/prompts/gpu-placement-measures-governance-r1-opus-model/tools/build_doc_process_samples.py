#!/usr/bin/env python3
"""门禁 doc-process-records 的 code-round-checklist 格的样本根：判几格看正文标题里第一个日期，那个日期由写正文的人定。

用法：build_doc_process_samples.py <输出目录>     写 <输出目录>/doc-process/<样本>/（.gate-cells 只点 code-round-checklist，research/prompts/ 下放代码轮正文）
样本：
  title-date-control            标题只有成稿日期（晚于第九格截止日），八格齐、缺「放的位置」：该红
  title-date-cited-earlier      同一份正文，标题里先写了它攻的那份判决的日期（早于第九格截止日）：只判八格
  title-date-before-checklist   一份标题里先写了检查单立起之前的日期、一格都没写；另一份九格齐：前一份整份不查
"""
import os
import sys

EIGHT_CELLS = ("走不到的理由二次查实", "类型与名字", "不兼容旧代码与旧格式", "不新建判定库", "所有任务多机可用", "测试优先多线程",
               "带 --item 的命令与项", "新崩溃流多罩了什么")
NINE_CELLS = EIGHT_CELLS + ("放的位置",)


def body(title, labels):
    return f"# {title}\n\n" + "".join(f"### 检查单：{label}\n\n成立：样本里的答案与证据，crates/demo/src/lib.rs:1。\n\n" for label in labels)


CASES = {
    "title-date-control": {
        "_placement-sample-code-r1-body.md": body("placement-sample-code-r1：样本代码轮第一轮正文（2026-12-19）", EIGHT_CELLS)},
    "title-date-cited-earlier": {
        "_placement-sample-code-r1-body.md": body("placement-sample-code-r1：攻 2026-11-18 判决的修补，样本代码轮第一轮正文（2026-12-19）", EIGHT_CELLS)},
    "title-date-before-checklist": {
        "_placement-sample-code-r1-body.md": body("placement-sample-code-r1：照 2026-11-05 那一版判决的修补（2026-12-19）", ()),
        "_placement-sample-code-r2-body.md": body("placement-sample-code-r2：样本代码轮第二轮正文（2026-12-19）", NINE_CELLS)},
}


def main(arguments):
    if len(arguments) != 1:
        print("用法：build_doc_process_samples.py <输出目录>", file=sys.stderr)
        return 2
    output = os.path.join(os.path.realpath(arguments[0]), "doc-process")
    if os.path.exists(output):
        print(f"  ✗ {output} 已存在", file=sys.stderr)
        return 2
    for case, bodies in CASES.items():
        prompts = os.path.join(output, case, "research", "prompts")
        os.makedirs(prompts)
        with open(os.path.join(output, case, ".gate-cells"), "w", encoding="utf-8") as handle:
            handle.write("code-round-checklist\n")
        for name, text in bodies.items():
            with open(os.path.join(prompts, name), "w", encoding="utf-8") as handle:
                handle.write(text)
    print(f"name=doc_process_samples_built count={len(CASES)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
