#!/usr/bin/env bash
# 红样本现场：data-names 那一格——一份实验源码里一条 match 分支放不下一行，cargo fmt 折成「=> {」「"名字"」「}」三行，串里带单字母 e；
# 同一份里 if / else 块里的串前面不是 =>，不是 match 分支，带缩写也不判。
set -e
mkdir -p scripts/experiment-apparatus
cat > scripts/experiment-apparatus/e1_demo.rs <<'RS'
fn stream_name(stream: u8) -> &'static str {
    match stream {
        0 => "new_pool_file_creation",
        1 => {
            "fixed_script_through_e_then_unmount"
        }
        _ => "tail_plain",
    }
}
fn width_label(wide: bool) -> &'static str {
    if wide { "wide_cnt" } else { "narrow_cnt" }
}
RS
git init -q . && git add -A
