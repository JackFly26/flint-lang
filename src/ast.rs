use std::fmt::{Debug, Error, Formatter, Display};
use std::collections::HashMap;
use std::path::Path;

// main Program struct
#[derive(Debug)]
pub struct Program<'a>(pub Vec<&'a Path>, pub Stack, pub Ops<'a>);

// stack of values
type Stack = Vec<Val>;

// ops in the ast
type Ops<'a> = Vec<(&'a str, OpType)>;
// environment as it's actually used in evaluation
type Env = HashMap<String, OpType>;

// operators can be builtin or user-defined
pub enum OpType {
	UserDefined(Stack),
	Builtin(Box<dyn Fn(&mut Stack, &Env)>)
}

// turns string into value
pub fn create_string(string: String) -> Val {
	let chars = string.chars();
	let mut res = vec![];
	for chr in chars {
		res.push(Val::Char(chr));
	}
	Val::Quote(res)
}

// i want to be able to print operators while debugging
impl Debug for OpType {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
    		use self::OpType::*;
		match self {
			UserDefined(stack) => write!(fmt, "UserDefined({:?})", stack),
			_ => write!(fmt, "[BUILTIN]")
		}
	}
}

// values that can be on the stack
#[derive(Clone, PartialEq)]
pub enum Val {
	Num(f64),
	Operator(String),
	Quote(Stack),
	Bool(bool),
	Char(char)
}

// allow for printing of values
impl Display for Val {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
		self.fmt_inter(false, fmt)
	}
}

// print values for debugging
impl Debug for Val {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
		self.fmt_inter(true, fmt)
	}
}

// evalute values
impl<'a> Val {
	fn fmt_inter(&self, debug: bool, fmt: &mut Formatter) -> Result<(), Error> {
		use self::Val::*;
		if self.is_string() {
    			if let Quote(stack) = self {
        			if debug {
					write!(fmt, "\"")?;
        			}
                		for val in stack {
                			if let Char(chr) = val {
                				write!(fmt, "{}", chr)?;
                			}
                		}
                		if debug {
					write!(fmt, "\"")?;
                		}
    			}
        		return Ok(())
		}
		match self {
			Num(num) => write!(fmt, "{:?}", num),
			Operator(op) => write!(fmt, "{:?}", op),
			Bool(bl) => write!(fmt, "{:?}", bl),
			Char(chr) => write!(fmt, "{:?}", chr),
			Quote(stack) => {
				write!(fmt, "[")?;
				let mut iter = stack.into_iter();
				if let Some(first) = iter.next() {
					write!(fmt, "{:?}", first)?;
				}
				for item in iter {
					write!(fmt, " {:?}", item)?;
				}
				write!(fmt, "]")
			}
		}
	}
	pub fn is_string(&self) -> bool{
		use self::Val::*;
		if let Quote(list) = self {
			for item in list {
				if let Char(_) = item {}
				else {
    					return false
				}
			}
			return true
		}
		false
	}
	pub fn eval(&self, stack: &mut Stack, env: &Env) {
    		use self::Val::*;
		match self {
			Operator(id) =>	match env.get(id) {
				Some(res) => match res {
					OpType::UserDefined(code) => {
						for val in code.iter() {
							val.clone().eval(stack, env);
						}
					}
					OpType::Builtin(fun) => fun(stack, env)
				}
				None => panic!("Undefined function {}.", id)
			},
			val => stack.push(val.clone())
		}
	}
}
