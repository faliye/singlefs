//! 状态数：1280 位的状态计数、256 位无符号数与混合进制的逐位加。

/// 态数的肢数：[`StateCount1280Bits`] 由这么多个 u128 拼成（「10 个 u128 拼起来」：按段登记的段的域、片起点、
/// 片尾与进度都用这个定宽整数，算术全部 checked，溢出这个位宽才拒跑）。
pub const STATE_COUNT_LIMBS: usize = 10;

/// 一个 [`StateCount1280Bits`] 存成字节的长度（小端，低肢在前）：事实表的片起点与判定库的进度记录按它存。
pub const STATE_COUNT_BYTES: usize = STATE_COUNT_LIMBS * 16;

/// 半肢（u64）的个数：乘、除小数时按半肢做，中间量装得进 u128。
const STATE_COUNT_HALF_LIMBS: usize = 2 * STATE_COUNT_LIMBS;

/// 一次十进制分块的除数 10^19（装得进 u64 的最大的 10 的幂）与它的位数。
const DECIMAL_CHUNK_DIVISOR: u64 = 10_000_000_000_000_000_000;
const DECIMAL_CHUNK_DIGITS: usize = 19;

/// 1280 位（[`STATE_COUNT_LIMBS`] 个 u128）的无符号态数：按段登记的段整域的态数（3^290 ≈ 2^460 装得下）、片起点、片尾、进度。
/// 算术全部 checked（交 `None` 就是越过 1280 位或减成负数），不饱和、不截断。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateCount1280Bits {
    /// 低肢在前。
    limbs: [u128; STATE_COUNT_LIMBS],
}

impl PartialOrd for StateCount1280Bits {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StateCount1280Bits {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.limbs.iter().rev().cmp(other.limbs.iter().rev())
    }
}

/// 一个 u128 的低 64 位。
fn low_sixty_four_bits(value: u128) -> u64 {
    u64::try_from(value & u128::from(u64::MAX)).expect("掩掉高 64 位之后装得进 u64")
}

/// 一个 u128 的高 64 位。
fn high_sixty_four_bits(value: u128) -> u64 {
    u64::try_from(value >> 64).expect("右移 64 位之后装得进 u64")
}

impl StateCount1280Bits {
    pub const ZERO: Self = Self {
        limbs: [0; STATE_COUNT_LIMBS],
    };

    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        limbs[0] = u128::from(value);
        Self { limbs }
    }

    /// 装得进 u64 就交回那个数。
    #[must_use]
    pub fn to_u64(&self) -> Option<u64> {
        if self.limbs[1..].iter().any(|limb| *limb != 0) {
            return None;
        }
        u64::try_from(self.limbs[0]).ok()
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|limb| *limb == 0)
    }

    fn halves(&self) -> [u64; STATE_COUNT_HALF_LIMBS] {
        let mut halves = [0u64; STATE_COUNT_HALF_LIMBS];
        for (limb_index, limb) in self.limbs.iter().enumerate() {
            halves[2 * limb_index] = low_sixty_four_bits(*limb);
            halves[2 * limb_index + 1] = high_sixty_four_bits(*limb);
        }
        halves
    }

    fn of_halves(halves: &[u64; STATE_COUNT_HALF_LIMBS]) -> Self {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            *limb =
                u128::from(halves[2 * limb_index]) | (u128::from(halves[2 * limb_index + 1]) << 64);
        }
        Self { limbs }
    }

    /// 两数之和；越过 1280 位交 `None`。
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        let mut carry = false;
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            let (sum, first_overflow) =
                self.limbs[limb_index].overflowing_add(other.limbs[limb_index]);
            let (sum, second_overflow) = sum.overflowing_add(u128::from(carry));
            *limb = sum;
            carry = first_overflow || second_overflow;
        }
        (!carry).then_some(Self { limbs })
    }

    /// 加一个 u64；越过 1280 位交 `None`。
    #[must_use]
    pub fn checked_add_u64(&self, addend: u64) -> Option<Self> {
        self.checked_add(&Self::from_u64(addend))
    }

    /// 两数之差；`other` 比它大交 `None`。
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        let mut borrow = false;
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            let (difference, first_borrow) =
                self.limbs[limb_index].overflowing_sub(other.limbs[limb_index]);
            let (difference, second_borrow) = difference.overflowing_sub(u128::from(borrow));
            *limb = difference;
            borrow = first_borrow || second_borrow;
        }
        (!borrow).then_some(Self { limbs })
    }

    /// 乘一个 u64；越过 1280 位交 `None`。
    #[must_use]
    pub fn checked_mul_u64(&self, multiplier: u64) -> Option<Self> {
        let mut halves = self.halves();
        let mut carry = 0u64;
        for half in &mut halves {
            // 半肢乘 u64 再加一个 u64 的进位：(2^64 − 1)^2 + 2^64 − 1 < 2^128，装得进 u128
            let product = u128::from(*half) * u128::from(multiplier) + u128::from(carry);
            *half = low_sixty_four_bits(product);
            carry = high_sixty_four_bits(product);
        }
        (carry == 0).then(|| Self::of_halves(&halves))
    }

    /// 除以一个正的 u64：交回（商，余数）。
    ///
    /// # Panics
    /// 除数是 0。
    #[must_use]
    pub fn divided_by_u64(&self, divisor: u64) -> (Self, u64) {
        assert_ne!(divisor, 0, "除数不许是 0");
        let mut halves = self.halves();
        let mut remainder = 0u64;
        for half in halves.iter_mut().rev() {
            // 余数小于除数、小于 2^64：余数左移 64 位再拼上这一个半肢，装得进 u128
            let dividend = (u128::from(remainder) << 64) | u128::from(*half);
            *half = u64::try_from(dividend / u128::from(divisor))
                .expect("被除数的高 64 位（余数）小于除数，商装得进 u64");
            remainder = u64::try_from(dividend % u128::from(divisor)).expect("余数小于除数");
        }
        (Self::of_halves(&halves), remainder)
    }

    /// 拆成（右移 64 位之后的高位，低 64 位）：崩溃放量分片的段的块键里块起点只有 8 字节，高位进计划哈希那一格。
    #[must_use]
    pub fn split_at_sixty_four_bits(&self) -> (Self, u64) {
        let halves = self.halves();
        let mut shifted = [0u64; STATE_COUNT_HALF_LIMBS];
        shifted[..STATE_COUNT_HALF_LIMBS - 1].copy_from_slice(&halves[1..]);
        (Self::of_halves(&shifted), halves[0])
    }

    /// 十进制，不带前导 0（0 写 `0`）：驱动行与拒跑句照它写，不写浮点、不写近似。
    #[must_use]
    pub fn decimal_text(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let mut chunks_from_the_lowest: Vec<u64> = Vec::new();
        let mut rest = *self;
        while !rest.is_zero() {
            let (quotient, chunk) = rest.divided_by_u64(DECIMAL_CHUNK_DIVISOR);
            chunks_from_the_lowest.push(chunk);
            rest = quotient;
        }
        let mut text = String::new();
        for (position, chunk) in chunks_from_the_lowest.iter().rev().enumerate() {
            if position == 0 {
                text.push_str(&chunk.to_string());
            } else {
                text.push_str(&format!("{chunk:0width$}", width = DECIMAL_CHUNK_DIGITS));
            }
        }
        text
    }

    /// 小端字节（低肢在前、每肢 16 字节小端）。
    #[must_use]
    pub fn to_le_bytes(&self) -> [u8; STATE_COUNT_BYTES] {
        let mut bytes = [0u8; STATE_COUNT_BYTES];
        for (limb_index, limb) in self.limbs.iter().enumerate() {
            bytes[limb_index * 16..(limb_index + 1) * 16].copy_from_slice(&limb.to_le_bytes());
        }
        bytes
    }

    #[must_use]
    pub fn from_le_bytes(bytes: &[u8; STATE_COUNT_BYTES]) -> Self {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        for (limb_index, limb) in limbs.iter_mut().enumerate() {
            *limb = u128::from_le_bytes(
                bytes[limb_index * 16..(limb_index + 1) * 16]
                    .try_into()
                    .expect("一肢 16 字节"),
            );
        }
        Self { limbs }
    }

    /// 按混合进位拆成各位（第 0 位最低，`radices` 是每一位的进制）；它不小于各进制之积（拆不下）交 `None`。
    ///
    /// # Panics
    /// 有一位的进制是 0。
    #[must_use]
    pub fn mixed_radix_digits(&self, radices: &[u8]) -> Option<Vec<u8>> {
        let mut rest = *self;
        let mut digits = Vec::with_capacity(radices.len());
        for radix in radices {
            let (quotient, digit) = rest.divided_by_u64(u64::from(*radix));
            digits.push(u8::try_from(digit).expect("一位上的数字小于它的进制，装得进 u8"));
            rest = quotient;
        }
        rest.is_zero().then_some(digits)
    }

    /// [`Self::mixed_radix_digits`] 的反方向；位数与进制数不同、有一位不小于它的进制、越过 1280 位交 `None`。
    #[must_use]
    pub fn of_mixed_radix_digits(digits: &[u8], radices: &[u8]) -> Option<Self> {
        if digits.len() != radices.len() {
            return None;
        }
        let mut value = Self::ZERO;
        for (digit, radix) in digits.iter().zip(radices).rev() {
            if digit >= radix {
                return None;
            }
            value = value
                .checked_mul_u64(u64::from(*radix))?
                .checked_add_u64(u64::from(*digit))?;
        }
        Some(value)
    }

    /// 左移 `bits` 位（乘 2^`bits`）；有一位移出 1280 位交 `None`（不截断）。
    #[must_use]
    pub fn checked_shift_left(&self, bits: usize) -> Option<Self> {
        if self.is_zero() {
            return Some(Self::ZERO);
        }
        if bits >= STATE_COUNT_LIMBS * 128 {
            return None;
        }
        let limb_shift = bits / 128;
        let bit_shift = bits % 128;
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        for (target, limb) in limbs.iter_mut().enumerate().skip(limb_shift) {
            let source = target - limb_shift;
            let mut value = self.limbs[source] << bit_shift;
            if bit_shift > 0 && source > 0 {
                value |= self.limbs[source - 1] >> (128 - bit_shift);
            }
            *limb = value;
        }
        let shifted = Self { limbs };
        (shifted.shifted_right(bits) == *self).then_some(shifted)
    }

    /// 右移 `bits` 位（除以 2^`bits`、向下取整）；移过 1280 位是 0。
    #[must_use]
    pub fn shifted_right(&self, bits: usize) -> Self {
        let limb_shift = bits / 128;
        let bit_shift = bits % 128;
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        for (target, limb) in limbs.iter_mut().enumerate() {
            let source = target + limb_shift;
            if source >= STATE_COUNT_LIMBS {
                break;
            }
            let mut value = self.limbs[source] >> bit_shift;
            if bit_shift > 0 && source + 1 < STATE_COUNT_LIMBS {
                value |= self.limbs[source + 1] << (128 - bit_shift);
            }
            *limb = value;
        }
        Self { limbs }
    }

    /// 读十进制数字串：只许 ASCII 数字、不空、不带前导 0（0 写 `0`，与 [`Self::decimal_text`] 一一对应）；
    /// 带符号、空白、别的字符或越过 1280 位交 `None`。
    #[must_use]
    pub fn from_decimal_text(text: &str) -> Option<Self> {
        let is_canonical_decimal = !text.is_empty()
            && text.bytes().all(|byte| byte.is_ascii_digit())
            && (text == "0" || !text.starts_with('0'));
        if !is_canonical_decimal {
            return None;
        }
        let mut value = Self::ZERO;
        for digit in text.bytes() {
            value = value
                .checked_mul_u64(10)?
                .checked_add_u64(u64::from(digit - b'0'))?;
        }
        Some(value)
    }

    /// 装得进双 128 位（[`Unsigned256Bits`]）就交回那个数。
    #[must_use]
    pub fn narrowed_to_256_bits(&self) -> Option<Unsigned256Bits> {
        if self.limbs[2..].iter().any(|limb| *limb != 0) {
            return None;
        }
        Some(Unsigned256Bits {
            high: self.limbs[1],
            low: self.limbs[0],
        })
    }
}

/// 双 128 位（两个 u128 拼成的 256 位）无符号整数：崩溃放量里登了分片段的流按份拆开之后的份号、份数与份内块序号
/// （块区间变量与驱动行都写十进制数字串）。算术全部 checked（交 `None` 就是越过 256 位或减成负数），不饱和、不截断。
/// 字段按高位在前排，派生出的大小次序就是数值的次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unsigned256Bits {
    high: u128,
    low: u128,
}

impl Unsigned256Bits {
    pub const ZERO: Self = Self { high: 0, low: 0 };

    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        Self {
            high: 0,
            low: u128::from(value),
        }
    }

    /// 装得进 u64 就交回那个数。
    #[must_use]
    pub fn to_u64(&self) -> Option<u64> {
        if self.high != 0 {
            return None;
        }
        u64::try_from(self.low).ok()
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.high == 0 && self.low == 0
    }

    /// 两数之和；越过 256 位交 `None`。
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let (low, carry) = self.low.overflowing_add(other.low);
        let high = self
            .high
            .checked_add(other.high)?
            .checked_add(u128::from(carry))?;
        Some(Self { high, low })
    }

    /// 两数之差；`other` 比它大交 `None`。
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        let (low, borrow) = self.low.overflowing_sub(other.low);
        let high = self
            .high
            .checked_sub(other.high)?
            .checked_sub(u128::from(borrow))?;
        Some(Self { high, low })
    }

    /// 放宽成 1280 位（[`StateCount1280Bits`]），值不变。
    #[must_use]
    pub fn widened(&self) -> StateCount1280Bits {
        let mut limbs = [0u128; STATE_COUNT_LIMBS];
        limbs[0] = self.low;
        limbs[1] = self.high;
        StateCount1280Bits { limbs }
    }

    /// 读十进制数字串（写法同 [`StateCount1280Bits::from_decimal_text`]）；越过 256 位交 `None`。
    #[must_use]
    pub fn from_decimal_text(text: &str) -> Option<Self> {
        StateCount1280Bits::from_decimal_text(text)?.narrowed_to_256_bits()
    }

    /// 十进制，不带前导 0（0 写 `0`）。
    #[must_use]
    pub fn decimal_text(&self) -> String {
        self.widened().decimal_text()
    }
}

/// 混合进位的各位上加一个 u64（第 0 位最低，逐位进位，不做宽整数除法：GPU 两个内核同一个算法、每一步只用 u32）：
/// 加完还有进位（越过各进制之积）交 `false`，这时各位是加到一半的样子、不能用。
///
/// # Panics
/// 位数与进制数不同；有一位的进制是 0。
#[must_use]
pub fn add_to_the_mixed_radix_digits(digits: &mut [u8], radices: &[u8], addend: u64) -> bool {
    assert_eq!(digits.len(), radices.len(), "每一位一个进制");
    let mut carry = addend;
    for (digit, radix) in digits.iter_mut().zip(radices) {
        if carry == 0 {
            break;
        }
        let radix = u64::from(*radix);
        // 进位先拆成「整进制的几倍」与余数，余数加到这一位上至多到 2 × 进制 − 2，不越过 u64
        let sum = u64::from(*digit) + carry % radix;
        *digit = u8::try_from(sum % radix).expect("一位上的数字小于它的进制，装得进 u8");
        carry = carry / radix + sum / radix;
    }
    carry == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 1280 位态数（位宽）：3^290 − 1（位置寻址树第三条流记录段 290 次原地写那一形的整域上界）按十进制逐字写出，
    /// 不饱和、不写成幂；拆成 290 位三进制每位都是 2、再拼回来是同一个数；装得进 u64 的照样取得出 u64。
    #[test]
    fn three_to_the_two_hundred_ninetieth_minus_one_is_written_in_decimal_exactly() {
        let mut power = StateCount1280Bits::from_u64(1);
        for _ in 0..290 {
            power = power
                .checked_mul_u64(3)
                .expect("3^290 ≈ 2^460，装得进 1280 位");
        }
        let domain = power
            .checked_sub(&StateCount1280Bits::from_u64(1))
            .expect("3^290 不小于 1");
        assert_eq!(
            domain.decimal_text(),
            "2318269218083089908234280468459894595445540070540355238386428052606631732586480323016481331702297098820479338186895528168360391078014797448"
        );
        assert_eq!(domain.to_u64(), None, "3^290 − 1 装不进 u64");
        let radices = vec![3u8; 290];
        let digits = domain.mixed_radix_digits(&radices).expect("小于 3^290");
        assert!(
            digits.iter().all(|digit| *digit == 2),
            "3^290 − 1 的每一位三进制都是 2"
        );
        assert_eq!(
            StateCount1280Bits::of_mixed_radix_digits(&digits, &radices),
            Some(domain)
        );
        assert_eq!(
            power.mixed_radix_digits(&radices),
            None,
            "3^290 本身拆不进 290 位三进制"
        );
        assert_eq!(
            StateCount1280Bits::from_u64(u64::MAX).to_u64(),
            Some(u64::MAX)
        );
        assert_eq!(
            StateCount1280Bits::from_u64(u64::MAX)
                .checked_add_u64(1)
                .map(|value| value.decimal_text()),
            Some("18446744073709551616".to_string())
        );
        assert_eq!(StateCount1280Bits::ZERO.decimal_text(), "0");
        assert_eq!(
            StateCount1280Bits::from_le_bytes(&domain.to_le_bytes()),
            domain,
            "小端字节往返不丢"
        );
    }

    /// 越过 1280 位交 `None`、不饱和：2^1279 再乘 2、1280 位全 1 再加 1、0 减 1 都拒。
    #[test]
    fn state_count_past_1280_bits_is_refused_not_saturated() {
        let mut highest_bit = StateCount1280Bits::from_u64(1);
        for _ in 0..1279 {
            highest_bit = highest_bit
                .checked_mul_u64(2)
                .expect("2^1279 装得进 1280 位");
        }
        assert_eq!(
            highest_bit.checked_mul_u64(2),
            None,
            "2^1280 装不进 1280 位"
        );
        let every_bit = highest_bit
            .checked_sub(&StateCount1280Bits::from_u64(1))
            .and_then(|below| below.checked_add(&highest_bit))
            .expect("2^1280 − 1 装得进");
        assert_eq!(
            every_bit.checked_add_u64(1),
            None,
            "1280 位全 1 再加 1 越过位宽"
        );
        assert_eq!(
            StateCount1280Bits::ZERO.checked_sub(&StateCount1280Bits::from_u64(1)),
            None
        );
        assert!(highest_bit > StateCount1280Bits::from_u64(u64::MAX));
    }

    /// 混合进位逐位进位地加：[2, 1]（三进制、两进制，值 2 + 3 = 5）加 1 得 [0, 0] 带出进位（越过 6），加 0 不动；
    /// [0, 0, 0] 加 13（三、二、三进制）得 [1, 0, 2]。
    #[test]
    fn adding_to_mixed_radix_digits_carries_digit_by_digit() {
        let mut digits = vec![2u8, 1];
        assert!(
            !add_to_the_mixed_radix_digits(&mut digits, &[3, 2], 1),
            "5 + 1 = 6 越过 3 × 2"
        );
        let mut unchanged = vec![2u8, 1];
        assert!(add_to_the_mixed_radix_digits(&mut unchanged, &[3, 2], 0));
        assert_eq!(unchanged, vec![2, 1]);
        let mut thirteen = vec![0u8, 0, 0];
        assert!(add_to_the_mixed_radix_digits(&mut thirteen, &[3, 2, 3], 13));
        assert_eq!(thirteen, vec![1, 0, 2], "13 = 1 + 3 × (0 + 2 × 2)");
    }
}
