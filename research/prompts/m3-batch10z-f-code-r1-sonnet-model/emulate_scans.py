import sys
src0 = open(sys.argv[1], encoding='utf-8').read()
NAME = "stop_is_requested_" + "by_the_environment"
def code_lines(source):
    start = source.index("#[cfg(test)]\nmod tests {")
    end = start + source[start:].index("\n}\n")
    text = source[:start] + source[end:]
    return [l for l in text.split('\n') if not l.lstrip().startswith('//')]
def test1(source):
    got = [l.strip() for l in code_lines(source) if NAME in l]
    want = ["pub fn " + NAME + "() -> bool {", "let stop_request = StopRequestRememberedInThisRun::new(&" + NAME + ");"]
    return got == want
def test2(source):
    lines = code_lines(source)
    arm = next(i for i, l in enumerate(lines) if l.strip() == "AfterARoundOfTheCards::StoppedOnRequest => {")
    body = []
    for l in lines[arm:]:
        if l.strip() == "break;": break
        body.append(l)
    body = "\n".join(body)
    return (".not_run_line_of_a_stop_after_an_out_of_memory(" in body and "PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE" in body and 'println!("{not_run_line}");' in body)
def mut(source, a, b):
    assert source.count(a) == 1, (a, source.count(a))
    return source.replace(a, b)
cases = [
 ("baseline (frozen)", src0),
 ("ROW25 mutation: 整趟收尾改回现读叫停文件", mut(src0, "    let stop_was_requested = gpu_memory_was_short || stop_is_requested();", "    let stop_was_requested = gpu_memory_was_short || stop_is_requested_by_the_environment();")),
 ("ROW26 mutation: 叫停收尾那一支不打头一句 OOM", mut(src0, "                if let Some(not_run_line) = rounds.not_run_line_of_a_stop_after_an_out_of_memory(\n                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,\n                ) {\n                    println!(\"{not_run_line}\");\n                }\n", "                // 变异\n")),
 ("REVERT-A 一轮收尾不问记住的叫停（常量 NotRequested）", mut(src0, "        let stop = stop_request.stop_of_the_run();\n        let run_card_indices", "        let stop = StopOfTheRun::NotRequested;\n        let run_card_indices")),
 ("REVERT-B 起下一张卡不看叫停（常量 NotRequested）", mut(src0, "        &work.blocks,\n        work.stop_of_the_cards(),\n    ) {", "        &work.blocks,\n        StopOfTheCards::NotRequested,\n    ) {")),
 ("REVERT-C 卡线程判块那一圈不看叫停", mut(src0, "        &|| work.stop_of_the_cards(),", "        &|| StopOfTheCards::NotRequested,")),
 ("REVERT-D 整趟收尾 stop_is_requested 闭包改成永远 false", mut(src0, "    let stop_is_requested = || stop_request.is_requested();", "    let stop_is_requested = || false;")),
 ("REVERT-E 打头一句 OOM 的那一支把 println 换成 eprintln", mut(src0, "                    println!(\"{not_run_line}\");\n                }\n                break;", "                    eprintln!(\"{not_run_line}\");\n                }\n                break;")),
]
for name, s in cases:
    print(f"{name}: scan1_pass={test1(s)} scan2_pass={test2(s)}")
