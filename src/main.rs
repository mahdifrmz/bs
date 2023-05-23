mod scanner;
#[cfg(test)]
mod tests;
mod text;
mod vm;

use compiler::CResult;
use scanner::Scanner;
use std::sync::Arc;
use text::Text;
use vm::{BVM, VM};

mod compiler;

#[derive(Default)]
struct BakhtScript {
    vm: BVM,
}

impl BakhtScript {
    fn reset(&mut self) {
        self.vm.reset();
    }
    fn load(&mut self, source: &str) -> CResult<()> {
        self.vm.reset();
        let text: Text = Arc::new(source.chars().collect());
        let scanner = Scanner::new(text.clone());
        let mut compiler = compiler::Compiler::new(text, scanner, BVM::default());
        compiler.compile()?;
        self.vm = compiler.vm();
        Ok(())
    }
    fn run(&mut self) {
        self.vm.run();
    }
}

fn main() {
    let mut bs = BakhtScript::default();
    bs.load(
        std::fs::read_to_string("./local/source.bs")
            .unwrap()
            .as_str(),
    )
    .unwrap();
}
