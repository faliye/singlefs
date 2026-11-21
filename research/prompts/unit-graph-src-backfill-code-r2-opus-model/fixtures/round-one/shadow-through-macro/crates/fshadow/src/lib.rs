macro_rules! call_through_the_module_path {
    ($function:ident) => {
        self::$function()
    };
}

macro_rules! call_inside_a_nested_function {
    ($body:block) => {{
        fn nested() -> u32 $body
        nested()
    }};
}

fn helper_total() -> u32 {
    7
}

pub fn entry_through_the_module_path() -> u32 {
    let helper_total = 1u32;
    call_through_the_module_path!(helper_total) + helper_total
}

pub fn entry_inside_a_nested_function() -> u32 {
    let helper_total = 1u32;
    helper_total + call_inside_a_nested_function!({ helper_total() })
}

pub fn entry_without_a_binding() -> u32 {
    helper_total()
}
