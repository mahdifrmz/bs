use crate::BakhtScript;

pub(crate) fn bakht_print(bakht: &mut BakhtScript) {
    match bakht.pop() {
        crate::BakhtValue::Function => println!("<function>"),
        crate::BakhtValue::Boolean(b) => println!("{}", b),
        crate::BakhtValue::Number(n) => println!("{}", n),
        crate::BakhtValue::Array => println!("[array]"),
        crate::BakhtValue::Nil => println!("nil"),
        crate::BakhtValue::String(s) => println!("{}", s),
    }
    bakht.push_nil();
}

pub(crate) fn bakht_push(bakht: &mut BakhtScript) {
    bakht.array_push();
    bakht.push_nil();
}
pub(crate) fn bakht_pop(bakht: &mut BakhtScript) {
    bakht.array_pop();
}
pub(crate) fn bakht_len(bakht: &mut BakhtScript) {
    bakht.array_len();
}
