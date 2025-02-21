fn main() {
    let mut s: [u32; 5] = [0; 5];
    if show(&s) {
        println!(" Empty array");
    }
    else {
        println!(" Non zero array");
    }
    s[3] = 55 << 16;
    if show(&s) {
        println!(" Empty array");
    }
    else {
        println!(" Non zero array");
    }
}

fn show(v:  &[u32]) -> bool {
    let mut result = true;
    for (i, &item) in v.iter().enumerate() {
        result = result && show_bytes(v[i]);
        println!("v[{i}] == {item} {result}");
        if !result {
            return result;
        }
    }
    return result;
}

fn show_bytes(b:  u32) -> bool {
    let mut result = true;
    let bytes = b.to_ne_bytes();
    for i in 0 .. bytes.len() {
        let b_ = bytes[i];
        result = result && (b_ == 0);
        println!("b[{i}] == {b_} {result}");
        if !result {
            return result;
        }
    }
    return result;
}
