use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("bla".to_string(), "bla".to_string());
    let a = "ble".to_string();
    let b = &a;
    map[*b];
    // TODO
}
