pub fn cycle(curr: &str, values: &[String]) -> String {
    for (i, v) in values.iter().enumerate() {
        if v == curr {
            let next_index = (i + 1) % values.len();
            return values[next_index].clone();
        }
    }
    
    if values.is_empty() {
        curr.to_string()
    } else {
        values[0].clone()
    }
}