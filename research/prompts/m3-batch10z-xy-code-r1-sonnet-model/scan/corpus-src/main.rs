#![allow(dead_code, unused, warnings)]
use std::io::BufRead;
use std::str::FromStr;
mod gpu { include!("gpu.rs"); pub fn tokens(code: &str) -> Vec<(usize, &str)> { rust_tokens_with_their_offsets(code) } }
mod crash {
    include!("crash.rs");
    /// (花括号层数，圆括号方括号层数：逐行累加的，产品函数在整份文本上的)
    pub fn infos(lines: &[&str]) -> (Vec<usize>, Vec<i64>, usize) {
        let read = lines_read_past_comments_and_literals(lines);
        let braces: Vec<usize> = read.iter().map(|r| r.brace_depth_at_the_start).collect();
        let mut running = 0i64; let mut parens = Vec::new();
        for r in &read {
            parens.push(running);
            for c in r.without_comments_and_literals.chars() {
                if matches!(c, '(' | '[') { running += 1; } else if matches!(c, ')' | ']') { running -= 1; }
            }
        }
        let joined = read.iter().map(|r| r.without_comments_and_literals.as_str()).collect::<Vec<_>>().join("\n");
        let product = parenthesis_and_bracket_depth_at_the_end_of(&joined);
        (braces, parens, product)
    }
}

/// 真值：每行起头处 (花括号层数, 圆括号方括号层数)，行号 1 起
fn truth(text: &str, line_count: usize) -> Option<(Vec<usize>, Vec<usize>)> {
    let stream = proc_macro2::TokenStream::from_str(text).ok()?;
    let mut brace = Vec::new(); let mut pb = Vec::new();
    fn walk(stream: proc_macro2::TokenStream, brace: &mut Vec<(usize, usize)>, pb: &mut Vec<(usize, usize)>) {
        let trees: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();
        for (i, tree) in trees.iter().enumerate() {
            if let proc_macro2::TokenTree::Group(g) = tree {
                // 文档注释（//! /// /*! /** */）被 proc_macro2 变成 #[doc = "…"]，括号组的跨度是整段注释：不是源码里的括号
                if g.delimiter() == proc_macro2::Delimiter::Bracket && i > 0 {
                    let mut inner = g.stream().into_iter();
                    let is_doc = matches!(inner.next(), Some(proc_macro2::TokenTree::Ident(id)) if id == "doc")
                        && matches!(inner.next(), Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '=')
                        && matches!(inner.next(), Some(proc_macro2::TokenTree::Literal(_)))
                        && inner.next().is_none();
                    let after_hash = matches!(&trees[i - 1], proc_macro2::TokenTree::Punct(p) if p.as_char() == '#')
                        || (i > 1 && matches!(&trees[i - 1], proc_macro2::TokenTree::Punct(p) if p.as_char() == '!') && matches!(&trees[i - 2], proc_macro2::TokenTree::Punct(p) if p.as_char() == '#'));
                    if is_doc && after_hash { continue; }
                }
                let span = (g.span_open().start().line, g.span_close().start().line);
                match g.delimiter() {
                    proc_macro2::Delimiter::Brace => brace.push(span),
                    proc_macro2::Delimiter::Parenthesis | proc_macro2::Delimiter::Bracket => pb.push(span),
                    proc_macro2::Delimiter::None => {}
                }
                walk(g.stream(), brace, pb);
            }
        }
    }
    walk(stream, &mut brace, &mut pb);
    let fold = |groups: Vec<(usize, usize)>| {
        let mut diff = vec![0i64; line_count + 3];
        for (o, c) in groups { if o < c { diff[o + 1] += 1; diff[c + 1] -= 1; } }
        let mut depth = 0i64; let mut out = Vec::new();
        for l in 1..=line_count { depth += diff[l]; out.push(depth as usize); }
        out
    };
    Some((fold(brace), fold(pb)))
}

/// 卡池分词那一份：每行起头处 (花括号层数, 圆括号方括号层数 或 None=负数即产品会 panic)
fn gpu_depths(text: &str, line_count: usize) -> (Vec<usize>, Vec<Option<usize>>) {
    let tokens = gpu::tokens(text);
    let mut line_starts = vec![0usize];
    for (i, b) in text.bytes().enumerate() { if b == b'\n' { line_starts.push(i + 1); } }
    let (mut b_out, mut p_out) = (Vec::new(), Vec::new());
    let (mut depth, mut pb, mut ti) = (0i64, 0i64, 0);
    let mut negative = false;
    for l in 0..line_count {
        let start = line_starts[l];
        while ti < tokens.len() && tokens[ti].0 < start {
            match tokens[ti].1 {
                "{" => depth += 1, "}" => depth -= 1,
                "(" | "[" => pb += 1,
                ")" | "]" => { pb -= 1; if pb < 0 { negative = true; } }
                _ => {}
            }
            ti += 1;
        }
        b_out.push(depth.max(0) as usize);
        p_out.push(if negative { None } else { Some(pb.max(0) as usize) });
    }
    (b_out, p_out)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut stats = std::collections::BTreeMap::<String, usize>::new();
    let stdin = std::io::stdin();
    for path in stdin.lock().lines() {
        let path = path.unwrap();
        let Ok(text) = std::fs::read_to_string(&path) else { *stats.entry("unreadable".into()).or_default() += 1; continue; };
        let lines: Vec<&str> = text.split('\n').collect();
        let Some((tb, tp)) = truth(&text, lines.len()) else { *stats.entry("proc_macro2_rejects".into()).or_default() += 1; continue; };
        *stats.entry("files_with_truth".into()).or_default() += 1;
        let crash_r = std::panic::catch_unwind(|| crash::infos(&lines));
        let gpu_r = std::panic::catch_unwind(|| gpu_depths(&text, lines.len()));
        let mut verdict = Vec::new();
        match &crash_r {
            Err(_) => verdict.push("crash_panic".to_string()),
            Ok((b, p, product)) => {
                if *b != tb { verdict.push("crash_brace_differs".into()); }
                let p_us: Vec<usize> = p.iter().map(|x| (*x).max(0) as usize).collect();
                if p.iter().any(|x| *x < 0) { verdict.push("crash_paren_negative_would_panic".into()); }
                if p_us != tp { verdict.push("crash_paren_bracket_differs".into()); }
                if *product != 0 { verdict.push("crash_paren_depth_at_end_nonzero".into()); }
            }
        }
        match &gpu_r {
            Err(_) => verdict.push("gpu_panic".to_string()),
            Ok((b, p)) => {
                if *b != tb { verdict.push("gpu_brace_differs".into()); }
                if p.iter().any(|x| x.is_none()) { verdict.push("gpu_paren_negative_would_panic".into()); }
                let p_us: Vec<usize> = p.iter().map(|x| x.unwrap_or(0)).collect();
                if p_us != tp { verdict.push("gpu_paren_bracket_differs".into()); }
            }
        }
        if let (Ok((_, cp, _)), Ok((_, gp))) = (&crash_r, &gpu_r) {
            let cu: Vec<usize> = cp.iter().map(|x| (*x).max(0) as usize).collect();
            let gu: Vec<usize> = gp.iter().map(|x| x.unwrap_or(0)).collect();
            if cu != gu { verdict.push("crash_vs_gpu_paren_differ".into()); }
        }
        for v in &verdict { *stats.entry(v.clone()).or_default() += 1; }
        if !verdict.is_empty() { println!("{path}: {}", verdict.join(" ")); }
    }
    for (k, v) in stats { println!("STAT {k} {v}"); }
}
