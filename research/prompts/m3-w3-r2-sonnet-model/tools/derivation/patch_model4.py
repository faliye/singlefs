#!/usr/bin/env python3
"""第四刀：加 replay 模式——按给定的操作序列在给定环长、给定臂上走一遍并逐步打印各候选红不红（用来在产品环长 24 上走一条具体历史）。
用法：patch_model4.py <输入 main.rs> <输出 main.rs>；model replay <臂 P|V1|V2|V3> <环长> <op,op,…>"""
import sys
src, out = sys.argv[1], sys.argv[2]
t = open(src, encoding="utf-8").read()
def once(old, new):
    global t
    n = t.count(old)
    if n != 1:
        sys.stderr.write(f"旧串命中 {n} 次：{old[:90]!r}\n"); sys.exit(3)
    t = t.replace(old, new)
once("""fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();""", """fn replay(arguments: &[String]) {
    let arm = match arguments[1].as_str() {
        "P" => Arm::Product,
        "V1" => Arm::RollbackTakesTheTargetsWatermark,
        "V2" => Arm::PublishDoesNotAddTheNewInodes,
        "V3" => Arm::RecoveryLowersTheWatermark,
        other => panic!("不认的臂 {other}"),
    };
    let ring_slots: usize = arguments[2].parse().expect("环长是整数");
    let names: Vec<&str> = arguments[3].split(',').collect();
    let config = Config { arm, mutation: Mutation::None, ring_slots, sweep_threshold: u64::MAX, longest: names.len() };
    let sequence: Vec<u8> = names
        .iter()
        .map(|name| {
            let index = Op::ALL.iter().position(|op| op.code() == *name).unwrap_or_else(|| panic!("不认的操作 {name}"));
            u8::try_from(index).expect("12 个")
        })
        .collect();
    println!("W3REPLAY {} ops={}", config_label(&config), arguments[3]);
    for line in render_trace(&config, &sequence) {
        // 只留判红名单，不重复打印整条根环（环长 24 时太长）：把 ring=[…] fixed=[…] 两段截掉。
        let cut = line.find(" ring=[").map_or(line.len(), |at| at);
        let red_at = line.find(" red=[");
        let mut shown = line[..cut].to_string();
        if let Some(at) = red_at {
            shown.push_str(&line[at..]);
        }
        println!("W3REPLAYSTEP {shown}");
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("replay") {
        replay(&arguments);
        return;
    }""")
open(out, "w", encoding="utf-8").write(t)
