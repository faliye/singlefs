// 草稿里的 WGSL 语法与类型校验：读一个 .wgsl 文件，经 naga 解析并校验。
// 走读内核的前言（Rust 一侧生成的常量）这里没有：解析报「没有定义的标识符」时现补一条 `const NAME: u32 = 1u;` 再试，补不动的错误照打。
use std::process::ExitCode;
fn main() -> ExitCode {
    let path = std::env::args().nth(1).expect("给一个 .wgsl 文件");
    let body = std::fs::read_to_string(&path).expect("读得了");
    let mut prelude = std::env::args().nth(2).map(|extra| std::fs::read_to_string(extra).expect("前言读得了")).unwrap_or_default();
    let mut stubs = 0usize;
    loop {
        let source = format!("{prelude}{body}");
        match naga::front::wgsl::parse_str(&source) {
            Ok(module) => {
                let mut validator = naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga::valid::Capabilities::all(),
                );
                println!("PARSE_OK functions={} entry_points={} stubbed_constants={stubs}", module.functions.len(), module.entry_points.len());
                // 前言常量是假值：模块级校验可能因常量下标越界等红，只当参考打出来
                match validator.validate(&module) {
                    Ok(_) => println!("VALIDATE_OK"),
                    Err(error) => println!("VALIDATE_INFO {}", error.emit_to_string(&source).lines().next().unwrap_or("")),
                }
                return ExitCode::SUCCESS;
            }
            Err(error) => {
                let text = error.emit_to_string(&source);
                let marker = "no definition in scope for identifier: `";
                if let Some(start) = text.find(marker) {
                    let rest = &text[start + marker.len()..];
                    let name = &rest[..rest.find('`').expect("反引号收尾")];
                    prelude.push_str(&format!("const {name}: u32 = 1u;\n")); println!("STUB {name}");
                    stubs += 1;
                    if stubs > 3000 {
                        println!("PARSE_ERROR 补了 3000 条常量还没完");
                        return ExitCode::from(1);
                    }
                    continue;
                }
                println!("PARSE_ERROR stubbed_constants={stubs} {text}");
                return ExitCode::from(1);
            }
        }
    }
}
