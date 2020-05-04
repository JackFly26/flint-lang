extern crate lalrpop;

fn main() {
    // codegen parser
    lalrpop::process_root().unwrap();
}
