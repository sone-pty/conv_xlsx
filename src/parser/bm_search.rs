use std::collections::HashMap;

pub fn bm_search(text: &str, pattern: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let n = text.len() as i32;
    let m = pattern.len() as i32;
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    if n == 0 || m == 0 {
        return positions;
    }
    let mut collection = HashMap::new();
    for (i, c) in pattern.iter().enumerate() {
        collection.insert(c, i as i32);
    }
    let mut shift: i32 = 0;
    while shift <= (n - m) {
        let mut j = m - 1;
        while j >= 0 && pattern[j as usize] == text[(shift + j) as usize] {
            j -= 1;
        }
        if j < 0 {
            positions.push(shift as usize);
            let add_to_shift = {
                if (shift + m) < n {
                    let c = text[(shift + m) as usize];
                    let index = collection.get(&c).unwrap_or(&-1);
                    m - index
                } else {
                    1
                }
            };
            shift += add_to_shift;
        } else {
            let c = text[(shift + j) as usize];
            let index = collection.get(&c).unwrap_or(&-1);
            let add_to_shift = std::cmp::max(1, j - index);
            shift += add_to_shift;
        }
    }
    positions
}