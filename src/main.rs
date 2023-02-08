use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("bla".to_string(), "bla".to_string());
    map.insert("ble".to_string(), "ble".to_string());
    let mut map2 = &map;

    map2.iter().map(|(c, f)| (c, f)).collect();
    // TODO
}
