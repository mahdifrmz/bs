use crate::BakhtScript;

pub(crate) fn bakh_print(bakht: &mut BakhtScript) {
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
