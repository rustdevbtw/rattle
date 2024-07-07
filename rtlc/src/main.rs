#![allow(dead_code, unused_variables)]

use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::error::Error;
use std::ops::{Add, Div, Mul, Rem, Sub};

/// A custom result type for the JIT compiler.
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// An enum to represent different types of values in the JIT compiler.
#[derive(Debug, Clone)]
pub enum JitValue {
    Int(i128),
    String(String),
    Float(f64), // Add more types as needed
}

impl PartialEq for JitValue {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (JitValue::Int(l), JitValue::Int(r)) => l == r,
            (JitValue::Float(l), JitValue::Float(r)) => l == r,
            (JitValue::String(l), JitValue::String(r)) => l == r,
            _ => false,
        }
    }
}

/// Implementing addition for JitValue.
impl Add for JitValue {
    type Output = JitValue;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (JitValue::Int(left), JitValue::Int(right)) => JitValue::Int(left + right),
            (JitValue::Float(left), JitValue::Float(right)) => JitValue::Float(left + right),
            _ => panic!("Unsupported operation: addition with non-matching types"),
        }
    }
}

/// Implementing subtraction for JitValue.
impl Sub for JitValue {
    type Output = JitValue;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (JitValue::Int(left), JitValue::Int(right)) => JitValue::Int(left - right),
            (JitValue::Float(left), JitValue::Float(right)) => JitValue::Float(left - right),
            _ => panic!("Unsupported operation: subtraction with non-matching types"),
        }
    }
}

/// Implementing multiplication for JitValue.
impl Mul for JitValue {
    type Output = JitValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (JitValue::Int(left), JitValue::Int(right)) => JitValue::Int(left * right),
            (JitValue::Float(left), JitValue::Float(right)) => JitValue::Float(left * right),
            _ => panic!("Unsupported operation: multiplication with non-matching types"),
        }
    }
}

/// Implementing division for JitValue.
impl Div for JitValue {
    type Output = JitValue;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (JitValue::Int(left), JitValue::Int(right)) => {
                if right == 0 {
                    panic!("Division by zero");
                }
                JitValue::Int(left / right)
            }
            (JitValue::Float(left), JitValue::Float(right)) => {
                if right == 0f64 {
                    panic!("Division by zero");
                }
                JitValue::Float(left / right)
            }
            _ => panic!("Unsupported operation: division with non-matching types"),
        }
    }
}

/// Implementing modulus for JitValue.
impl Rem for JitValue {
    type Output = JitValue;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (JitValue::Int(left), JitValue::Int(right)) => {
                if right == 0 {
                    panic!("Division by zero");
                }
                JitValue::Int(left % right)
            }
            (JitValue::Float(left), JitValue::Float(right)) => {
                if right == 0f64 {
                    panic!("Division by zero");
                }
                JitValue::Float(left % right)
            }
            _ => panic!("Unsupported operation: modulus with non-matching types"),
        }
    }
}

/// Metadata for JIT variables.
pub struct JitMeta {
    is_mut: bool,
}

/// A struct representing the JIT compiler.
struct JITCompiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    var_types: HashMap<&'static str, (JitMeta, JitValue)>,
    should_execute: bool,
}

impl<'ctx> JITCompiler<'ctx> {
    /// Creates a new JITCompiler instance.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native target");

        let module = context.create_module(module_name);
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::Aggressive)
            .expect("Failed to create JIT execution engine");

        Self {
            context,
            module,
            execution_engine,
            var_types: HashMap::new(),
            should_execute: true, // Start with execution enabled
        }
    }

    /// Declares a constant variable.
    pub fn decl_const(&mut self, name: &'static str, value: JitValue) {
        self.var_types
            .insert(name, (JitMeta { is_mut: false }, value));
    }

    /// Declares an immutable variable.
    pub fn decl_var(&mut self, name: &'static str, value: JitValue) {
        self.var_types
            .insert(name, (JitMeta { is_mut: false }, value));
    }

    /// Declares a mutable variable.
    pub fn decl_var_mut(&mut self, name: &'static str, value: JitValue) {
        self.var_types
            .insert(name, (JitMeta { is_mut: true }, value));
    }

    /// Assigns a new value to a variable.
    pub fn assign_var(&mut self, name: &'static str, value: JitValue) {
        if let Some((m, entry)) = self.var_types.get_mut(name) {
            if m.is_mut {
                *entry = value;
            } else {
                panic!("Variable '{}' is immutable!", name);
            }
        } else {
            panic!("Variable '{}' not found", name);
        }
    }

    /// Gets a reference to a variable.
    pub fn get(&self, name: &'static str) -> Result<Option<&JitValue>> {
        Ok(self.var_types.get(name).map(|s| &s.1))
    }

    /// Gets a cloned value of a variable.
    pub fn get_auto(&self, name: &'static str) -> Result<JitValue> {
        self.get(name)?
            .ok_or_else(|| format!("Variable '{}' not found", name).into())
            .map(|v| v.clone())
    }

    /// Implements a switch-case-like structure.
    pub fn switch(
        &self,
        name: &'static str,
        cases: Vec<(JitValue, JitValue)>,
        default: JitValue,
    ) -> Result<JitValue> {
        let actual = self.get_auto(name)?;
        for case in cases {
            if actual == case.0 {
                return Ok(case.1);
            }
        }
        Ok(default)
    }

    /// Runs a JIT-compiled function.
    pub fn run_function(&self, jit_fn: JitFunction<unsafe extern "C" fn() -> i32>) -> Result<i32> {
        Ok(unsafe { jit_fn.call() })
    }

    /// Gets the execution engine.
    pub fn get_execution_engine(&self) -> &ExecutionEngine<'ctx> {
        &self.execution_engine
    }
}

/// A macro to convert JIT values to strings.
#[macro_export]
macro_rules! typed {
    ($jit_compiler:expr, $name:expr) => {{
        match $jit_compiler.get_auto($name) {
            Ok(value) => match value {
                JitValue::Int(int_value) => int_value.to_string(),
                JitValue::String(string_value) => string_value.clone(),
                JitValue::Float(float_value) => float_value.to_string(),
                // Add more cases for other types as needed
            },
            Err(err) => panic!("Error: {}", err),
        }
    }};
}

#[tokio::main]
async fn main() -> Result<()> {
    let context = Context::create();

    // Create JIT compiler instance
    let mut jit_compiler = JITCompiler::new(&context, "jit_module");

    // Declare a mutable integer variable
    jit_compiler.decl_var_mut("val", JitValue::Int(10));

    // Assign a new value to the mutable variable
    jit_compiler.assign_var("val", JitValue::Float(19f64));

    // Declare a constant float variable
    jit_compiler.decl_const("_", JitValue::Float(20f64));

    // Perform addition on the variable
    jit_compiler.assign_var("val", jit_compiler.get_auto("val")? + JitValue::Float(1f64));

    // Print the variable value
    println!("{}", typed!(jit_compiler, "val"));

    // Print the result of adding two variables
    println!(
        "{:?}",
        jit_compiler.get_auto("val")? + jit_compiler.get_auto("_")?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let context = Context::create();
        let mut jit_compiler = JITCompiler::new(&context, "jit_test");

        jit_compiler.decl_var_mut("a", JitValue::Int(10));
        jit_compiler.assign_var("a", JitValue::Int(20));

        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Int(20));
        jit_compiler.assign_var("a", jit_compiler.get_auto("a").unwrap() + JitValue::Int(5));
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Int(25));

        jit_compiler.assign_var("a", JitValue::Float(15.0));
        jit_compiler.assign_var(
            "a",
            jit_compiler.get_auto("a").unwrap() + JitValue::Float(5.5),
        );
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Float(20.5));

        jit_compiler.decl_var_mut("b", JitValue::Float(3.3));
        jit_compiler.assign_var(
            "b",
            jit_compiler.get_auto("b").unwrap() + JitValue::Float(4.7),
        );
        assert_eq!(jit_compiler.get_auto("b").unwrap(), JitValue::Float(8.0));
    }

    #[test]
    fn test_subtraction() {
        let context = Context::create();
        let mut jit_compiler = JITCompiler::new(&context, "jit_test");

        jit_compiler.decl_var_mut("a", JitValue::Int(10));
        jit_compiler.assign_var("a", JitValue::Int(20));
        jit_compiler.assign_var("a", jit_compiler.get_auto("a").unwrap() - JitValue::Int(5));
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Int(15));

        jit_compiler.assign_var("a", JitValue::Float(15.0));
        jit_compiler.assign_var(
            "a",
            jit_compiler.get_auto("a").unwrap() - JitValue::Float(5.5),
        );
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Float(9.5));

        jit_compiler.decl_var_mut("b", JitValue::Float(20.0));
        jit_compiler.assign_var(
            "b",
            jit_compiler.get_auto("b").unwrap() - JitValue::Float(4.7),
        );
        assert_eq!(jit_compiler.get_auto("b").unwrap(), JitValue::Float(15.3));
    }

    #[test]
    fn test_multiplication() {
        let context = Context::create();
        let mut jit_compiler = JITCompiler::new(&context, "jit_test");

        jit_compiler.decl_var_mut("a", JitValue::Int(5));
        jit_compiler.assign_var("a", jit_compiler.get_auto("a").unwrap() * JitValue::Int(2));
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Int(10));

        jit_compiler.assign_var("a", JitValue::Float(1.5));
        jit_compiler.assign_var(
            "a",
            jit_compiler.get_auto("a").unwrap() * JitValue::Float(2.0),
        );
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Float(3.0));

        jit_compiler.decl_var_mut("b", JitValue::Float(4.0));
        jit_compiler.assign_var(
            "b",
            jit_compiler.get_auto("b").unwrap() * JitValue::Float(2.5),
        );
        assert_eq!(jit_compiler.get_auto("b").unwrap(), JitValue::Float(10.0));
    }

    #[test]
    fn test_division() {
        let context = Context::create();
        let mut jit_compiler = JITCompiler::new(&context, "jit_test");

        jit_compiler.decl_var_mut("a", JitValue::Int(20));
        jit_compiler.assign_var("a", jit_compiler.get_auto("a").unwrap() / JitValue::Int(2));
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Int(10));

        jit_compiler.assign_var("a", JitValue::Float(9.0));
        jit_compiler.assign_var(
            "a",
            jit_compiler.get_auto("a").unwrap() / JitValue::Float(3.0),
        );
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Float(3.0));

        jit_compiler.decl_var_mut("b", JitValue::Float(20.0));
        jit_compiler.assign_var(
            "b",
            jit_compiler.get_auto("b").unwrap() / JitValue::Float(4.0),
        );
        assert_eq!(jit_compiler.get_auto("b").unwrap(), JitValue::Float(5.0));
    }

    #[test]
    fn test_modulus() {
        let context = Context::create();
        let mut jit_compiler = JITCompiler::new(&context, "jit_test");

        jit_compiler.decl_var_mut("a", JitValue::Int(21));
        jit_compiler.assign_var("a", jit_compiler.get_auto("a").unwrap() % JitValue::Int(4));
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Int(1));

        jit_compiler.assign_var("a", JitValue::Float(10.5));
        jit_compiler.assign_var(
            "a",
            jit_compiler.get_auto("a").unwrap() % JitValue::Float(3.0),
        );
        assert_eq!(jit_compiler.get_auto("a").unwrap(), JitValue::Float(1.5));

        jit_compiler.decl_var_mut("b", JitValue::Float(20.0));
        jit_compiler.assign_var(
            "b",
            jit_compiler.get_auto("b").unwrap() % JitValue::Float(6.0),
        );
        assert_eq!(jit_compiler.get_auto("b").unwrap(), JitValue::Float(2.0));
    }

    #[test]
    fn test_switch() {
        let context = Context::create();
        let mut jit_compiler = JITCompiler::new(&context, "jit_test");

        jit_compiler.decl_var_mut("case", JitValue::Int(1));

        let cases = vec![
            (JitValue::Int(0), JitValue::String("Zero".to_string())),
            (JitValue::Int(1), JitValue::String("One".to_string())),
            (JitValue::Int(2), JitValue::String("Two".to_string())),
        ];

        let result = jit_compiler
            .switch(
                "case",
                cases.clone(),
                JitValue::String("Default".to_string()),
            )
            .unwrap();
        assert_eq!(result, JitValue::String("One".to_string()));

        jit_compiler.assign_var("case", JitValue::Int(0));
        let result = jit_compiler
            .switch(
                "case",
                cases.clone(),
                JitValue::String("Default".to_string()),
            )
            .unwrap();
        assert_eq!(result, JitValue::String("Zero".to_string()));

        jit_compiler.assign_var("case", JitValue::Int(2));
        let result = jit_compiler
            .switch("case", cases, JitValue::String("Default".to_string()))
            .unwrap();
        assert_eq!(result, JitValue::String("Two".to_string()));
    }

    #[test]
    fn test_typed_macro() {
        let context = Context::create();
        let mut jit_compiler = JITCompiler::new(&context, "jit_test");

        jit_compiler.decl_var("test_int", JitValue::Int(42));
        jit_compiler.decl_var("test_float", JitValue::Float(3.14));
        jit_compiler.decl_var("test_string", JitValue::String("hello".to_string()));

        assert_eq!(typed!(jit_compiler, "test_int"), "42");
        assert_eq!(typed!(jit_compiler, "test_float"), "3.14");
        assert_eq!(typed!(jit_compiler, "test_string"), "hello");
    }
}
