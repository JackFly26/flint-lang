use std::collections::HashMap;
use std::io::{self, Write};

use crate::ast::{Val, OpType, self};

// register a builtin
fn builtin(name: &str, env: &mut HashMap<String, OpType>, fun: impl Fn(&mut Vec<Val>, &HashMap<String, OpType>) + 'static) {
    env.insert(name.to_string(), OpType::Builtin(Box::new(fun)));
}

// AST string to String
fn from_string(string: Val) -> String {
	let mut res = String::new();
	if let Val::Quote(vec) = string {
		for val in vec.into_iter() {
    			if let Val::Char(chr) = val {
				res.push(chr);
    			}
		}
	}
	res
}

// append 2 stacks after the point they differ at
fn merge_stacks(old: &mut Vec<Val>, new: &Vec<Val>) {
	for i in 0..new.len() {
    		if let Some(original) = old.get(i) {
        		if &new[i] != original {
        			old.push(new[i].clone());
        		}
    		} else {
        		old.push(new[i].clone())
    		}
	}
}

// eval quoted code
fn eval_quote(quote: &Vec<Val>, stack: &mut Vec<Val>, env: &HashMap<String, OpType>) {
	for val in quote.into_iter() {
		val.eval(stack, env);
	}
}

// get number off the stack
fn expect_num(stack: &mut Vec<Val>) -> f64 {
	let val = expect_val(stack);
	if let Val::Num(a) = val {
		a
	} else {
		panic!("Expecting number, found {}.", val);
	}
}

// get char off the stack
fn expect_char(stack: &mut Vec<Val>) -> char {
	let val = expect_val(stack);
	if let Val::Char(a) = val {
    		a
	} else {
		panic!("Expecting number, found {}.", val);
	}
}

// get bool off the stack
fn expect_bool(stack: &mut Vec<Val>) -> bool {
	let val = expect_val(stack);
	if let Val::Bool(a) = val {
		a
	} else {
		panic!("Expecting boolean, found {}.", val);
	}
}

// get quote off the stack
fn expect_quote(stack: &mut Vec<Val>) -> Vec<Val> {
	let val = expect_val(stack);
	if let Val::Quote(a) = val {
		a
	} else {
		panic!("Expecting quote, found {}", val);
	}
}

//get any value off the stack
fn expect_val(stack: &mut Vec<Val>) -> Val {
	if let Some(a) = stack.pop() {
    		a
	} else {
		panic!("Expecting value, found empty stack.")
	}
}

// register builtins
pub fn builtins() -> HashMap<String, OpType> {
	use crate::ast::Val::*;
	let mut env = HashMap::new();
	// remove value from the stack
	builtin("pop", &mut env, |stack: &mut Vec<Val>, _| {
		expect_val(stack);
	});
	// duplicate a value on the stack
	builtin("dup", &mut env, |stack: &mut Vec<Val>, _| {
		let val = expect_val(stack);
		stack.push(val.clone());
		stack.push(val);
    	});
    	// swap 2 values on the stack [x, y] -> [y, x]
    	builtin("swap", &mut env, |stack: &mut Vec<Val>, _| {
		let y = expect_val(stack);
		let x = expect_val(stack);
		stack.push(y);
		stack.push(x);
    	});
    	// move 2nd and 3rd value forward, top value goes to 3rd position [x, y, z] -> [z, x, y]
    	builtin("rollup", &mut env, |stack: &mut Vec<Val>, _| {
		let z = expect_val(stack);
		let y = expect_val(stack);
		let x = expect_val(stack);
		stack.push(z);
		stack.push(x);
		stack.push(y);
    	});
    	// opposite of rollup, [x, y, z] -> [y, z, x]
    	builtin("rolldown", &mut env, |stack: &mut Vec<Val>, _| {
		let z = expect_val(stack);
		let y = expect_val(stack);
		let x = expect_val(stack);
		stack.push(y);
		stack.push(z);
		stack.push(x);
        });
        // swap top and 3rd value, [x, y, z] -> [z, y, x]
        builtin("rotate", &mut env, |stack: &mut Vec<Val>, _| {
		let x = expect_val(stack);
		let y = expect_val(stack);
		let z = expect_val(stack);
		stack.push(z);
		stack.push(y);
		stack.push(x);
        });
        // conditionally evaluate quotes based off boolean
        builtin("if", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let if_false = expect_quote(stack);
		let if_true = expect_quote(stack);
		let cond = expect_bool(stack);
		if cond {
			eval_quote(&if_true, stack, &env);
		} else {
			eval_quote(&if_false, stack, &env);
		}
	});
	// conditionally evaluate quotes based off predicate
	builtin("ifte", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let if_false = expect_quote(stack);
		let if_true = expect_quote(stack);
		let cond_quote = expect_quote(stack);
		eval_quote(&cond_quote, stack, env);
		let cond = expect_bool(stack);
		if cond {
			eval_quote(&if_true, stack, env);
		} else {
			eval_quote(&if_false, stack, env);
		}
    	});
    	// do n times
    	builtin("times", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let times = expect_num(stack) as i64;
		let quote = expect_quote(stack);
		for _ in 0..times {
			eval_quote(&quote, stack, env);
		}
        });
    	// bitwise operations
	builtin("or", &mut env, |stack: &mut Vec<Val>, _| {
		let x = expect_bool(stack);
		let y = expect_bool(stack);
		stack.push(Bool(x || y));
    	});
	builtin("xor", &mut env, |stack: &mut Vec<Val>, _| {
		let x = expect_bool(stack);
		let y = expect_bool(stack);
		stack.push(Bool(x ^ y));
    	});
	builtin("and", &mut env, |stack: &mut Vec<Val>, _| {
		let x = expect_bool(stack);
		let y = expect_bool(stack);
		stack.push(Bool(x && y));
    	});
    	builtin("not", &mut env, |stack: &mut Vec<Val>, _| {
		let x = expect_bool(stack);
		stack.push(Bool(!x));
    	});
    	// numerical operations
	builtin("+", &mut env, |stack: &mut Vec<Val>, _| {
    		let second = expect_num(stack);
    		let first = expect_num(stack);
		stack.push(Num(first + second));
	});
	builtin("-", &mut env, |stack: &mut Vec<Val>, _| {
    		let second = expect_num(stack);
    		let first = expect_num(stack);
		stack.push(Num(first - second));
	});
	builtin("*", &mut env, |stack: &mut Vec<Val>, _| {
    		let second = expect_num(stack);
    		let first = expect_num(stack);
		stack.push(Num(first * second));
	});
	builtin("/", &mut env, |stack: &mut Vec<Val>, _| {
    		let second = expect_num(stack);
    		let first = expect_num(stack);
		stack.push(Num(first / second));
	});
	builtin("mod", &mut env, |stack: &mut Vec<Val>, _| {
    		let second = expect_num(stack);
    		let first = expect_num(stack);
		stack.push(Num(first % second));
	});
	builtin("div", &mut env, |stack: &mut Vec<Val>, _| {
    		let second = expect_num(stack);
    		let first = expect_num(stack);
		stack.push(Num((first as i64 / second as i64) as f64));
	});
	// x -> x - 1
	builtin("pred", &mut env, |stack: &mut Vec<Val>, _| {
		let num = expect_num(stack);
		stack.push(Num(num - 1.));
    	});
    	// x -> x + 1
	builtin("succ", &mut env, |stack: &mut Vec<Val>, _| {
		let num = expect_num(stack);
		stack.push(Num(num + 1.));
    	});
    	// [[x, y, z, ...]] -> [x, [y, z, ...]]
    	builtin("uncons", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let list = expect_quote(stack);
		if let Some((head, tail)) = list.split_first() {
			head.clone().eval(stack, env);
			stack.push(Quote(tail.to_vec()));
		} else {
			panic!("Called uncons on empty list.")
		}
        });
        // opposite of uncons
        builtin("cons", &mut env, |stack: &mut Vec<Val>, _| {
		let mut list = expect_quote(stack).clone();
		let elem = expect_val(stack);
		list.insert(0, elem);
		stack.push(Quote(list));
        });
        builtin("~", &mut env, |stack: &mut Vec<Val>, _| {
		let mut second = expect_quote(stack).clone();
		let mut first = expect_quote(stack).clone();
		first.append(&mut second);
		stack.push(Quote(first));
        });
        // check for empty list
        builtin("null", &mut env, |stack: &mut Vec<Val>, _| {
		let list = expect_quote(stack);
		stack.push(Bool(list.is_empty()));
	});
	// relational operations
	builtin(">=", &mut env, |stack: &mut Vec<Val>, _| {
		let right = expect_num(stack);
		let left = expect_num(stack);
		stack.push(Bool(left >= right));
    	});
	builtin(">", &mut env, |stack: &mut Vec<Val>, _| {
		let right = expect_num(stack);
		let left = expect_num(stack);
		stack.push(Bool(left > right));
    	});
	builtin("<=", &mut env, |stack: &mut Vec<Val>, _| {
		let right = expect_num(stack);
		let left = expect_num(stack);
		stack.push(Bool(left <= right));
    	});
	builtin("<", &mut env, |stack: &mut Vec<Val>, _| {
		let right = expect_num(stack);
		let left = expect_num(stack);
		stack.push(Bool(left < right));
    	});
	builtin("==", &mut env, |stack: &mut Vec<Val>, _| {
		let right = expect_val(stack);
		let left = expect_val(stack);
		stack.push(Bool(left == right));
	});
	builtin("!=", &mut env, |stack: &mut Vec<Val>, _| {
		let right = expect_val(stack);
		let left = expect_val(stack);
		stack.push(Bool(left != right));
	});
	// check for type of value
	builtin("num?", &mut env, |stack: &mut Vec<Val>, _| {
		if let Num(_) = expect_val(stack) {
    			stack.push(Bool(true));
		} else {
    			stack.push(Bool(false));
		}
    	});
	builtin("bool?", &mut env, |stack: &mut Vec<Val>, _| {
		if let Bool(_) = expect_val(stack) {
    			stack.push(Bool(true));
		} else {
    			stack.push(Bool(false));
		}
    	});
	builtin("quote?", &mut env, |stack: &mut Vec<Val>, _| {
		if let Quote(_) = expect_val(stack) {
    			stack.push(Bool(true));
		} else {
    			stack.push(Bool(false));
		}
    	});
    	// wrap in a quote
    	builtin("quote", &mut env, |stack: &mut Vec<Val>, _| {
		let val = expect_val(stack);
		stack.push(Quote(vec![val]));
        });
    	// evaluate a quote
    	builtin("unquote", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let quote = expect_quote(stack);
		eval_quote(&quote, stack, env);
        });
        // run quote on stack, preserving top value
        builtin("dip", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let quote = expect_quote(stack);
		let val = expect_val(stack);
		eval_quote(&quote, stack, env);
		stack.push(val);
	});
	// run quote, accepting no arguments
	builtin("nullary", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let quote = expect_quote(stack);
		let mut new_stack = stack.clone();
		eval_quote(&quote, &mut new_stack, env);
		merge_stacks(stack, &new_stack);
	});
	// run quote, accepting 1 argument
	builtin("unary", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let quote = expect_quote(stack);
		let mut new_stack = stack.clone();
		eval_quote(&quote, &mut new_stack, env);
		let _ = expect_val(stack);
		merge_stacks(stack, &new_stack);
	});
	// run quote, accepting n arguments
	builtin("n-ary", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let n = expect_num(stack) as i64;
		let quote = expect_quote(stack);
		let mut new_stack = stack.clone();
		eval_quote(&quote, &mut new_stack, env);
		for _ in 0..n {
			let _ = expect_val(stack);
		}
		merge_stacks(stack, &new_stack);
	});
	// run 2 quotes with a value at top of stack
	builtin("cleave", &mut env, |stack: &mut Vec<Val>, env: &HashMap<String, OpType>| {
		let q2 = expect_quote(stack);
		let q1 = expect_quote(stack);
		let el = expect_val(stack);
		let mut stack_2 = stack.clone();
		stack.push(el.clone());
		stack_2.push(el);
		eval_quote(&q1, stack, env);
		eval_quote(&q2, &mut stack_2, env);
		merge_stacks(stack, &stack_2);
    	});
    	// type conversions
    	builtin("->str", &mut env, |stack: &mut Vec<Val>, _| {
		let val = expect_val(stack);
		stack.push(ast::create_string(format!("{}", val)));
        });
        builtin("str->num", &mut env, |stack: &mut Vec<Val>, _| {
		let string = expect_val(stack);
		if string.is_string() {
			stack.push(Num(from_string(string).parse().unwrap()));
		} else {
			panic!("Expecting string, found {}.", string);
		}
        });
	// get unicode value of a char and vice versa
        builtin("ord", &mut env, |stack: &mut Vec<Val>, _| {
		let chr = expect_char(stack);
		let mut buffer = [0; 2];
		let res = chr.encode_utf16(&mut buffer).iter().fold(0 as u32, |acc, next| acc << 16 | *next as u32);
		stack.push(Num(res as f64));
		
        });
        builtin("num->char", &mut env, |stack: &mut Vec<Val>, _| {
		use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
		let num = expect_num(stack) as u32;
		let units = [((num & 0xffff0000) >> 16) as u16, (num & 0x0000ffff) as u16];
		let mut str = decode_utf16(units.iter().cloned()).map(|r| r.unwrap_or(REPLACEMENT_CHARACTER)).collect::<String>();
		stack.push(Char(str.pop().unwrap()));
        });
	// get user input
	builtin("input", &mut env, |stack: &mut Vec<Val>, _| {
    		let _ = io::stdout().flush();
		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Reading input failed.");
		let _ = input.pop();
		stack.push(ast::create_string(input));
 	});
 	//print a value
 	builtin("print", &mut env, |stack: &mut Vec<Val>, _| {
		let val = expect_val(stack);
		print!("{}", val);
 	});
 	// print a value with a newline
	builtin("println", &mut env, |stack: &mut Vec<Val>, _| {
		let val = expect_val(stack);
		println!("{}", val);
    	});
    	// debug-print a value
    	builtin("debug", &mut env, |stack: &mut Vec<Val>, _| {
		let val = expect_val(stack);
		println!("{:?}", val);
        });
    	// print entire stack
    	builtin("printstack", &mut env, |stack: &mut Vec<Val>, _| {
		println!("{:?}", Quote(stack.clone()));
        });
    	// booleans
	builtin("true", &mut env, |stack: &mut Vec<Val>, _| {
		stack.push(Bool(true));
    	});
    	builtin("false", &mut env, |stack: &mut Vec<Val>, _| {
		stack.push(Bool(false));
    	});
	env
}
