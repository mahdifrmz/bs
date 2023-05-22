mod scanner;
#[cfg(test)]
mod tests;
mod text;
mod vm;

use compiler::CResult;
use scanner::Scanner;
use std::sync::Arc;
use text::Text;
use vm::VM;

mod compiler;

#[derive(Default)]
struct BakhtScript {}

impl BakhtScript {
    fn run(&self, source: &str) -> CResult<()> {
        let text: Text = Arc::new(source.chars().collect());
        let scanner = Scanner::new(text.clone());
        let vm = vm::BVM {};
        let mut compiler = compiler::Compiler::new(text, scanner, vm);
        compiler.compile()?;
        let mut vm = compiler.vm();
        vm.run();
        Ok(())
    }
}

fn main() {
    let bs = BakhtScript::default();
    bs.run(
        std::fs::read_to_string("./local/source.bs")
            .unwrap()
            .as_str(),
    )
    .unwrap();
}
