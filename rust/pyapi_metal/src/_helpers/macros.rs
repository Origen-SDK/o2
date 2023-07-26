#[macro_export]
macro_rules! alias_method_apply_to_set {
    ($m: expr, $cls: tt, $basename: tt) => {{
        let cls = $m.getattr($cls)?;
        let func = cls.getattr(concat!("apply_", $basename))?;
        cls.setattr(concat!("set_", $basename), func)?;
    }};
}
