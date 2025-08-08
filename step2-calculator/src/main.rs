use clap::{Parser, Subcommand};
use anyhow::Result;
use std::io::{self, Write};

// カスタムエラー型の定義
#[derive(thiserror::Error, Debug)]
pub enum CalcError {
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
    
    #[error("Number parsing error: {0}")]
    ParseError(#[from] std::num::ParseFloatError),
    
    #[error("Unknown operation: {0}")]
    UnknownOperation(String),
}

// CLIコマンド構造体
#[derive(Parser)]
#[command(name = "calc-cli")]
#[command(about = "A simple calculator CLI tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Basic arithmetic operations
    #[command(alias = "a")]
    Add {
        /// First number
        a: f64,
        /// Second number
        b: f64,
    },
    
    /// Subtract two numbers
    #[command(alias = "s")]
    Subtract {
        /// First number
        a: f64,
        /// Second number to subtract
        b: f64,
    },
    
    /// Multiply two numbers
    #[command(alias = "m")]
    Multiply {
        /// First number
        a: f64,
        /// Second number
        b: f64,
    },
    
    /// Divide two numbers
    #[command(alias = "d")]
    Divide {
        /// Dividend
        a: f64,
        /// Divisor
        b: f64,
    },
    
    /// Calculate power (a^b)
    #[command(alias = "p")]
    Power {
        /// Base
        base: f64,
        /// Exponent
        exp: f64,
    },
    
    /// Calculate square root
    #[command(alias = "sqrt")]
    SquareRoot {
        /// Number to calculate square root
        number: f64,
    },
    
    /// Evaluate mathematical expression
    #[command(alias = "e")]
    Eval {
        /// Mathematical expression (e.g., "2 + 3 * 4")
        expression: String,
    },
    
    /// Interactive mode
    #[command(alias = "i")]
    Interactive,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { a, b }) => {
            let result = add(a, b)?;
            println!("{} + {} = {}", a, b, result);
        }
        
        Some(Commands::Subtract { a, b }) => {
            let result = subtract(a, b)?;
            println!("{} - {} = {}", a, b, result);
        }
        
        Some(Commands::Multiply { a, b }) => {
            let result = multiply(a, b)?;
            println!("{} * {} = {}", a, b, result);
        }
        
        Some(Commands::Divide { a, b }) => {
            let result = divide(a, b)?;
            println!("{} / {} = {}", a, b, result);
        }
        
        Some(Commands::Power { base, exp }) => {
            let result = power(base, exp)?;
            println!("{}^{} = {}", base, exp, result);
        }
        
        Some(Commands::SquareRoot { number }) => {
            let result = square_root(number)?;
            println!("√{} = {}", number, result);
        }
        
        Some(Commands::Eval { expression }) => {
            let result = evaluate_expression(&expression)?;
            println!("{} = {}", expression, result);
        }
        
        Some(Commands::Interactive) => {
            run_interactive_mode()?;
        }
        
        None => {
            println!("No command provided. Use --help for usage information.");
            println!("Quick examples:");
            println!("  calc-cli add 10 5");
            println!("  calc-cli eval \"2 + 3 * 4\"");
            println!("  calc-cli interactive");
        }
    }

    Ok(())
}

// 基本的な算術関数
fn add(a: f64, b: f64) -> Result<f64, CalcError> {
    let result = a + b;
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::InvalidExpression("Result overflow".to_string()));
    }
    Ok(result)
}

fn subtract(a: f64, b: f64) -> Result<f64, CalcError> {
    let result = a - b;
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::InvalidExpression("Result overflow".to_string()));
    }
    Ok(result)
}

fn multiply(a: f64, b: f64) -> Result<f64, CalcError> {
    let result = a * b;
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::InvalidExpression("Result overflow".to_string()));
    }
    Ok(result)
}

fn divide(a: f64, b: f64) -> Result<f64, CalcError> {
    if b == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    
    let result = a / b;
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::InvalidExpression("Result overflow".to_string()));
    }
    Ok(result)
}

fn power(base: f64, exp: f64) -> Result<f64, CalcError> {
    if base < 0.0 && exp.fract() != 0.0 {
        return Err(CalcError::InvalidExpression(
            "Cannot calculate non-integer power of negative number".to_string()
        ));
    }
    
    let result = base.powf(exp);
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::InvalidExpression("Result overflow or invalid".to_string()));
    }
    Ok(result)
}

fn square_root(number: f64) -> Result<f64, CalcError> {
    if number < 0.0 {
        return Err(CalcError::InvalidExpression(
            "Cannot calculate square root of negative number".to_string()
        ));
    }
    
    Ok(number.sqrt())
}

// 簡単な式評価（四則演算のみ）
fn evaluate_expression(expr: &str) -> Result<f64, CalcError> {
    let expr = expr.replace(" ", ""); // 空白を削除
    
    // 非常にシンプルな実装：優先順位を考慮した解析
    // 実際のプロジェクトでは、より堅牢なパーサーを使用することを推奨
    
    // 加算と減算を処理
    if let Some(pos) = expr.rfind('+') {
        let left = evaluate_expression(&expr[..pos])?;
        let right = evaluate_expression(&expr[pos + 1..])?;
        return add(left, right).map_err(|e| e.into());
    }
    
    if let Some(pos) = expr.rfind('-') {
        // マイナス記号が先頭にある場合は負の数として処理
        if pos == 0 {
            let number = evaluate_expression(&expr[1..])?;
            return Ok(-number);
        }
        let left = evaluate_expression(&expr[..pos])?;
        let right = evaluate_expression(&expr[pos + 1..])?;
        return subtract(left, right).map_err(|e| e.into());
    }
    
    // 乗算と除算を処理
    if let Some(pos) = expr.rfind('*') {
        let left = evaluate_expression(&expr[..pos])?;
        let right = evaluate_expression(&expr[pos + 1..])?;
        return multiply(left, right).map_err(|e| e.into());
    }
    
    if let Some(pos) = expr.rfind('/') {
        let left = evaluate_expression(&expr[..pos])?;
        let right = evaluate_expression(&expr[pos + 1..])?;
        return divide(left, right).map_err(|e| e.into());
    }
    
    // 数値として解析
    expr.parse::<f64>()
        .map_err(|_| CalcError::InvalidExpression(expr.to_string()))
}

// インタラクティブモード
fn run_interactive_mode() -> Result<()> {
    println!("Calculator Interactive Mode");
    println!("Enter mathematical expressions or 'quit' to exit");
    println!("Examples: 2 + 3, 10 / 2, sqrt 16");
    
    loop {
        print!("calc> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }
        
        if input == "help" {
            print_help();
            continue;
        }
        
        // 特別なコマンドを処理
        if input.starts_with("sqrt ") {
            let number_str = input.strip_prefix("sqrt ").unwrap();
            match number_str.parse::<f64>() {
                Ok(number) => {
                    match square_root(number) {
                        Ok(result) => println!("√{} = {}", number, result),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Err(_) => println!("Error: Invalid number format"),
            }
            continue;
        }
        
        // 式として評価
        match evaluate_expression(input) {
            Ok(result) => println!("{} = {}", input, result),
            Err(e) => println!("Error: {}", e),
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("Available operations:");
    println!("  Basic: +, -, *, /");
    println!("  Special: sqrt <number>");
    println!("  Commands: help, quit, exit");
    println!("Examples:");
    println!("  2 + 3");
    println!("  10 / 2");
    println!("  sqrt 16");
    println!("  -5 + 3");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        assert_eq!(add(2.0, 3.0).unwrap(), 5.0);
        assert_eq!(subtract(5.0, 3.0).unwrap(), 2.0);
        assert_eq!(multiply(4.0, 3.0).unwrap(), 12.0);
        assert_eq!(divide(10.0, 2.0).unwrap(), 5.0);
    }

    #[test]
    fn test_division_by_zero() {
        assert!(matches!(divide(5.0, 0.0), Err(CalcError::DivisionByZero)));
    }

    #[test]
    fn test_square_root() {
        assert_eq!(square_root(16.0).unwrap(), 4.0);
        assert_eq!(square_root(9.0).unwrap(), 3.0);
        assert!(square_root(-1.0).is_err());
    }

    #[test]
    fn test_power() {
        assert_eq!(power(2.0, 3.0).unwrap(), 8.0);
        assert_eq!(power(5.0, 2.0).unwrap(), 25.0);
        assert!(power(-2.0, 0.5).is_err()); // 負数の非整数乗
    }

    #[test]
    fn test_expression_evaluation() {
        assert_eq!(evaluate_expression("2 + 3").unwrap(), 5.0);
        assert_eq!(evaluate_expression("10 - 4").unwrap(), 6.0);
        assert_eq!(evaluate_expression("3 * 4").unwrap(), 12.0);
        assert_eq!(evaluate_expression("15 / 3").unwrap(), 5.0);
        assert_eq!(evaluate_expression("2 + 3 * 4").unwrap(), 14.0); // 演算子優先順位
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(evaluate_expression("-5").unwrap(), -5.0);
        assert_eq!(evaluate_expression("-5 + 3").unwrap(), -2.0);
    }

    #[test]
    fn test_error_cases() {
        assert!(evaluate_expression("5 / 0").is_err());
        assert!(evaluate_expression("abc").is_err());
        assert!(evaluate_expression("").is_err());
    }
}
