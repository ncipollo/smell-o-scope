fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn classify(value: i32) -> &'static str {
    if value < 0 {
        "negative"
    } else if value == 0 {
        "zero"
    } else {
        "positive"
    }
}
