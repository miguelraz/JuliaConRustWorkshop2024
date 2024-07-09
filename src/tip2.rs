mod tip2 {
    
    pub enum Color {
        Red,
        Green,
        Blue
    }

    // Tip 2: expr.match rewrite
    pub fn foo() {
        let color = Color::Red;
        let some_n = Some(15);
        let n = 16;
    
        (n % 3, n % 5);

        color;
            
        some_n;
    }
}
