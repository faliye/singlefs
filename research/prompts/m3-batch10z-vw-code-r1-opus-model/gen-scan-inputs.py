#!/usr/bin/env python3
"""K4：从冻结副本两份 checker 档源文件造扫源码的攻法输入：gen-scan-inputs.py <冻结副本根> <输出目录>。
每份写出去，再打印它测试模块头两行（代码里、字面上）各出现几次，供核。"""
import sys
from pathlib import Path

frozen, out = Path(sys.argv[1]), Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
crash = (frozen / "crates/singlefs-checker-tier/src/crash_amplification.rs").read_text()
gpu = (frozen / "crates/singlefs-checker-tier/src/gpu_unit_checks.rs").read_text()
opening = "#[cfg(test)]\nmod tests {"
assert crash.count(opening) == 1 and gpu.count(opening) == 1

# ---- crash_amplification.rs：收叫停那一扫（text_outside_the_tests_module 行粒度 + stop_request_shapes_outside_the_pinned_type） ----
layer_plain = 'pub fn an_extra_layer_never_asking(first_input: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { let _ = stop_is_requested; first_input == 0 }\n'
# 上一轮打中的：块注释里写测试模块头两行；新代码应当照旧切到真测试模块。
fake_block = "/*\n#[cfg(test)]\nmod tests {\n*/\n"
# 新攻法：测试模块头两行各写在行注释里（// 开头），插一层新层。
fake_line = "// #[cfg(test)]\n// mod tests {\n"
# 新攻法：把头两行写进一个跨行字符串字面量，插一层新层。
fake_string = 'const NOTE: &str = "\n#[cfg(test)]\nmod tests {\n";\n'
# 新攻法：新层收叫停写成 `&(dyn Fn() -> bool + Sync)` 之外的形（Send、Box<dyn>、&mut、fn 指针、impl、别名），看 shape 扫抓不抓。
shapes = {
    "send": "pub fn atk_send(first_input: u8, stop_is_requested: &(dyn Fn() -> bool + Send + Sync)) -> bool { let _ = stop_is_requested; first_input == 0 }\n",
    "boxed": "pub fn atk_boxed(first_input: u8, stop_is_requested: Box<dyn Fn() -> bool + Sync>) -> bool { let _ = stop_is_requested; first_input == 0 }\n",
    "fnptr": "pub fn atk_fnptr(first_input: u8, stop_is_requested: fn() -> bool) -> bool { let _ = stop_is_requested; first_input == 0 }\n",
    "mut": "pub fn atk_mut(first_input: u8, stop_is_requested: &mut dyn FnMut() -> bool) -> bool { let _ = stop_is_requested; first_input == 0 }\n",
    "impltrait": "pub fn atk_impl(first_input: u8, stop_is_requested: impl Fn() -> bool) -> bool { let _ = stop_is_requested; first_input == 0 }\n",
}
# 新攻法：收叫停那种闭包形写进字面量 / 注释里——不该当成一层。
shape_in_string = 'const ATK_NOTE: &str = "fn atk_in_a_string(stop: Fn() -> bool) {}";\n'
shape_in_comment = "// fn atk_in_a_comment(stop: Fn() -> bool) {}\n"
# 新攻法：收叫停闭包形用 cr 原始字符串 / C 字符串包起来（分词应当认它们）。
shape_cr = 'const ATK_CR: &str = cr#"Fn() -> bool"#;\n'

def before_opening(text, insert):
    return text.replace(opening, insert + "\n" + opening, 1)

crash_inputs = {
    "c0-frozen.rs": crash,
    "c1-new-layer-plain.rs": before_opening(crash, layer_plain),
    "c2-commented-opening-then-layer.rs": before_opening(crash, fake_block + layer_plain),
    "c3-line-commented-opening-then-layer.rs": before_opening(crash, fake_line + layer_plain),
    "c4-string-opening-then-layer.rs": before_opening(crash, fake_string + layer_plain),
    "c5-shape-in-string.rs": before_opening(crash, shape_in_string),
    "c6-shape-in-comment.rs": before_opening(crash, shape_in_comment),
    "c7-shape-in-cr-string.rs": before_opening(crash, shape_cr),
}
for name, layer in shapes.items():
    crash_inputs[f"c-shape-{name}.rs"] = before_opening(crash, layer)

# ---- gpu_unit_checks.rs：卡池那一扫（code_before_the_tests_module 词粒度 + 卡池模块扫） ----
pool_anchor = "    mod card_pool {\n"
assert gpu.count(pool_anchor) == 1
gpu_inputs = {
    "g0-frozen.rs": gpu,
    # 上一轮打中过的：块注释里写测试模块头两行（新代码 code_before_the_tests_module 应当照旧切）
    "g1-commented-opening-in-the-pool.rs": gpu.replace(pool_anchor, pool_anchor + "        /*\n#[cfg(test)]\nmod tests {\n        */\n", 1),
    # use wgpu; 引的就是外部 crate 自己（新代码 source != name 应当放）
    "g2-use-wgpu-itself.rs": gpu.replace(pool_anchor, pool_anchor + "        use wgpu;\n", 1),
    # 块里的 use（函数体，没有钉住那两段的签名）：新代码 does_the_scope… 应当放
    "g3-block-rename-no-pin.rs": gpu.replace(pool_anchor, pool_anchor + "        fn atk_helper() -> u8 {\n            use core as std;\n            std::u8::MAX\n        }\n", 1),
    # impl ... + use<'a>：新代码 use 后面紧跟 < 的不当 use，应当放
    "g4-impl-use-capture.rs": gpu.replace(pool_anchor, pool_anchor + "        fn atk_capture<'a>(x: &'a u8) -> impl Fn() -> &'a u8 + use<'a> { move || x }\n", 1),
    # 属性里的表达式宏 #[doc = include_str!(...)]：新代码看前一个词是 ] 但在 = 后面——token_before 判……这里前一个词是 =，应当放
    "g5-attribute-expr-macro.rs": gpu.replace(pool_anchor, pool_anchor + '        #[doc = include_str!("../../../README.md")]\n        pub fn atk_doc() {}\n', 1),
    # 常量初始值里的表达式宏 const _: () = assert!(...);：前一个词是 =，应当放
    "g6-const-expr-macro.rs": gpu.replace(pool_anchor, pool_anchor + "        const _: () = assert!(true);\n", 1),
    # r#mod 原始标识符拼成关键字：应当当名字、不当关键字（不拒）
    "g7-raw-ident-keyword.rs": gpu.replace(pool_anchor, pool_anchor + "        fn r#mod() -> u8 { 0 }\n", 1),
    # 攻法：块里有钉住那两段签名时，块里的 use 应当被罩到、判红（阳性对照新规则没放过头）
    "g8-block-rename-with-pin-signature.rs": gpu.replace(pool_anchor, pool_anchor + "        fn atk_with_pin() {\n            use core as std;\n            // fn take_in_the_devices_found_idle\n            let _ = \"fn take_in_the_devices_found_idle\";\n        }\n", 1),
    # 攻法：挂 #[cfg] 的 use 引进按名字调的函数（新规则应当判红）
    "g9-cfg-use-pinned-function.rs": gpu.replace(pool_anchor, pool_anchor + "        #[cfg(test)]\n        use super::super::device_found_idle_by_a_poll;\n", 1),
    # 攻法：条目层真宏（前一个词是 ; 或 }）应当照旧判红
    "g10-item-level-macro.rs": gpu.replace(pool_anchor, pool_anchor + "        thread_local! { static ATK: u8 = 0; }\n", 1),
}

for name, text in {**crash_inputs, **gpu_inputs}.items():
    (out / name).write_text(text)
    print(name, "opening_literal_count", text.count(opening))
