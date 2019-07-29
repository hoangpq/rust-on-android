#[macro_export]
macro_rules! js_object {
    ( $($key:expr => $value:expr), *) => {{
        let object = $crate::empty_object();
        $(
            object.set_from_raw($key, $value);
        )*
        object
    }};
}
