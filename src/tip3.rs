mod tip3 {
    // Tip 3: Hover on types
    fn hover() -> i32 {
        (0..10)
            .filter(|x| x % 2 == 0)
            .map(|x| x * x)
            .sum::<i32>()
    }
}
