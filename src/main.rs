#[macro_use]
extern crate lalrpop_util;

mod ast;
mod builtins;

use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::hash::Hash;

lalrpop_mod!(pub lang); // syntesized by LALRPOP

// convert ast ops to an environment with the builtins
fn ops_to_env(builtins: HashMap<String, ast::OpType>, ops: Vec<(&str, ast::OpType)>) -> HashMap<String, ast::OpType> {
	let mut env = HashMap::new();
	for (k, v) in builtins {
		env.insert(k, v);
	}
	for (k, v) in ops.into_iter() {
		env.insert(k.to_string(), v);
	}
	env
}

fn vec_to_hash_set<T: Hash + Eq>(vec: Vec<T>) -> HashSet<T> {
	let mut set = HashSet::new();
	for elem in vec.into_iter() {
		set.insert(elem);
	}
	set
}

fn resolve_imports(paths: HashSet<&Path>, builtins: &mut HashMap<String, ast::OpType>, passed: HashSet<&Path>) {
    for path in paths.iter() {
        let mut file = File::open(&path).expect("Error importing file.");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Error importing file.");
        let ast::Program(imports, _, ops) = lang::ProgramParser::new().parse(&contents).expect("Error importing file.");
	let imports_set: HashSet<&Path> = vec_to_hash_set(imports);
	let union: HashSet<&Path> = passed.union(&imports_set).map(|a: &&Path| *a).collect();
	let diff: HashSet<&Path> = imports_set.difference(&paths).map(|a: &&Path| *a).collect();
        resolve_imports(diff, builtins, union);
        for (k, v) in ops.into_iter() {
            builtins.insert(k.to_string(), v);
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    	// parse source code
	let parser = lang::ProgramParser::new();
	let mut file = File::open(Path::new("./main.flint"))?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	let ast::Program(mut imports, start, ops) = parser.parse(&contents).expect("Error parsing file.");
	// add builtins and imports
	imports.insert(0, Path::new("./prelude.flint"));
	let mut builtins = builtins::builtins();
	resolve_imports(imports.into_iter().collect(), &mut builtins, HashSet::new());
	let env = ops_to_env(builtins, ops);
	// eval code
	let mut stack = vec![];
	for val in start.into_iter() {
		val.eval(&mut stack, &env);
	}
	Ok(())
}
