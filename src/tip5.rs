#[allow(unused_must_use)]
mod tip5 {
    // Tip 5: Recursive Macro expand
    macro_rules! add {
        ($a:expr, $b:expr) => {
            $a + $b
        };
    }

    pub fn recursive_expand() {
        let x = 4;
        add!(add!(x, x), 1);
    }
}
