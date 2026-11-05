//! 门禁样本：impl 里的 spec const Window::SPEC_WIDTH 与自由的 spec const SPEC_HEIGHT（照里程碑四收口 2026-10-29 在真 Verus 0.2026.09.20 上量的那一份探针 crate：
//! 整 crate 验出 5 条 = exec fn 1 + 带值的 const 2 + spec const 2；可点名函数表列 impl&%0::SPEC_WIDTH 与 SPEC_HEIGHT，点 *::SPEC_WIDTH 与 SPEC_HEIGHT 各验出 1 条）。
//! spec const 与别的带值的 const 一样成项、一样点名，列项退 0、成项 5 个；改前 spec const 不成项，只列 3 个、各项之和比整 crate 少 2 条。
use vstd::prelude::*;

verus! {

pub struct Window(pub u64);

impl Window {
    pub const NARROW: Self = Self(1);

    pub spec const SPEC_WIDTH: u64 = 4;
}

pub spec const SPEC_HEIGHT: u64 = 3;

pub const PLAIN_HEIGHT: u64 = 3;

pub fn height() -> (result: u64)
    ensures
        result == SPEC_HEIGHT,
{
    PLAIN_HEIGHT
}

} // verus!
