macro_rules! format {
    ($function:ident) => {
        self::$function()
    };
}

macro_rules! vec {
    ($body:block) => {{
        fn nested() -> u32 $body
        nested()
    }};
}

fn helper_total() -> u32 {
    7
}

pub fn entry_through_a_macro_named_format() -> u32 {
    let helper_total = 1u32;
    format!(helper_total) + helper_total
}

pub fn entry_through_a_macro_named_vec() -> u32 {
    let helper_total = 1u32;
    helper_total + vec!({ helper_total() })
}

pub fn entry_without_a_binding() -> u32 {
    helper_total()
}

fn main() {
    println!("runtime entry_through_a_macro_named_format={}", entry_through_a_macro_named_format());
    println!("runtime entry_through_a_macro_named_vec={}", entry_through_a_macro_named_vec());
}
