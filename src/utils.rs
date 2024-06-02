pub fn get_line(lines: &Vec<&str>, start: usize) -> (usize, usize) {
    let mut line = 0;
    let mut count = 0;
    for (i, l) in lines.iter().enumerate() {
        if start - count < l.len() {
            line = i;
            break;
        }
        count += l.len() + 1;
    }

    (line, count)
}
