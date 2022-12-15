use serde_json::Value;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let json: Value = serde_json::from_str(INPUT).expect("ill-formed json");

    let pt1 = fold_json(&json, false);
    println!("part 1: {pt1}");

    let pt2 = fold_json(&json, true);
    println!("part 2: {pt2}");
}

fn fold_json(json: &Value, skip_red: bool) -> f64 {
    match json {
        Value::Null => 0.0,
        Value::Bool(b) => *b as u8 as f64,
        Value::Number(n) => n.as_f64().expect("should be a number"),
        Value::String(_) => 0.0,
        Value::Array(a) => a.iter().map(|v| fold_json(v, skip_red)).sum(),
        Value::Object(o) => {
            if skip_red && o.values().any(|v| v.as_str() == Some("red")) {
                0.0
            } else {
                o.values().map(|v| fold_json(v, skip_red)).sum()
            }
        }
    }
}
