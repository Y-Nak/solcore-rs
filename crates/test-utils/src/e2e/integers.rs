//! Shared integer E2E matrix and mathematical oracle for both backends.
//!
//! Arithmetic inputs and results travel as raw words, while the separate ABI
//! matrix checks typed codecs with independently encoded inputs and outputs.

use num_bigint::BigInt;
use std::{fmt::Write as _, fs, path::Path};

use super::{E2eFailure, FailureKind, e2e_enabled};

mod abi;

/// Runs the shared matrix through a backend's existing fixture pipeline.
pub fn run(run_fixture: impl Fn(&Path) -> Result<(), E2eFailure>) -> Result<(), E2eFailure> {
    run_matrix(fixtures, run_fixture)
}

/// Exercises integer ABI codecs using independently encoded calldata and results.
pub fn run_abi(run_fixture: impl Fn(&Path) -> Result<(), E2eFailure>) -> Result<(), E2eFailure> {
    run_matrix(abi::fixtures, run_fixture)
}

fn run_matrix(
    fixtures: impl FnOnce() -> Vec<(String, String)>,
    run_fixture: impl Fn(&Path) -> Result<(), E2eFailure>,
) -> Result<(), E2eFailure> {
    if !e2e_enabled() {
        return Ok(());
    }
    let io_failure =
        |error| E2eFailure::new(FailureKind::Tooling, format!("integer fixture: {error}"));
    let root = tempfile::tempdir().map_err(io_failure)?;
    for (name, source) in fixtures() {
        let directory = root.path().join(&name);
        fs::create_dir(&directory).map_err(io_failure)?;
        let path = directory.join("main.sol");
        fs::write(&path, source).map_err(io_failure)?;
        run_fixture(&path).map_err(|failure| {
            E2eFailure::new(failure.kind, format!("{name}: {}", failure.message))
        })?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Operand {
    Constant(i32),
    Min,
    Max,
    BeforeMax,
    Half,
}

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Not,
    Compare,
    Bounds,
}

const OPERATIONS: [Operation; 11] = [
    Operation::Add,
    Operation::Sub,
    Operation::Mul,
    Operation::Div,
    Operation::Mod,
    Operation::And,
    Operation::Or,
    Operation::Xor,
    Operation::Not,
    Operation::Compare,
    Operation::Bounds,
];

impl Operation {
    fn name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Not => "not",
            Self::Compare => "cmp",
            Self::Bounds => "bounds",
        }
    }

    fn expression(self) -> &'static str {
        match self {
            Self::Add => "x + y",
            Self::Sub => "x - y",
            Self::Mul => "x * y",
            Self::Div => "x / y",
            Self::Mod => "x % y",
            Self::And => "x & y",
            Self::Or => "x | y",
            Self::Xor => "x ^ y",
            Self::Not => "~x",
            Self::Compare | Self::Bounds => unreachable!("these operations have custom bodies"),
        }
    }

    fn arity(self) -> usize {
        match self {
            Self::Bounds => 0,
            Self::Not => 1,
            _ => 2,
        }
    }

    fn inputs(self, signed: bool) -> &'static [(Operand, Operand)] {
        use Operand::{BeforeMax, Constant as N, Half, Max, Min};
        match (signed, self) {
            (true, Self::Add) => &[
                (N(1), N(2)),
                (N(-5), N(3)),
                (Max, N(0)),
                (Min, Max),
                (Max, N(1)),
                (Min, N(-1)),
            ],
            (true, Self::Sub) => &[
                (N(3), N(5)),
                (N(-3), N(-5)),
                (N(-1), Max),
                (Min, N(0)),
                (Min, N(1)),
                (Max, N(-1)),
            ],
            (true, Self::Mul) => &[
                (N(-7), N(6)),
                (N(-7), N(-6)),
                (Min, N(1)),
                (N(0), Min),
                (Max, N(2)),
                (Min, N(-1)),
            ],
            (true, Self::Div) => &[
                (N(-7), N(2)),
                (N(7), N(-2)),
                (N(-8), N(-2)),
                (Min, N(1)),
                (Min, N(-1)),
                (N(5), N(0)),
            ],
            (true, Self::Mod) => &[
                (N(7), N(3)),
                (N(-7), N(3)),
                (N(7), N(-3)),
                (Min, N(-1)),
                (N(5), N(0)),
            ],
            (true, Self::And) => &[(N(-1), N(15)), (N(-16), N(12))],
            (true, Self::Or) => &[(N(-16), N(5)), (Min, Max)],
            (true, Self::Xor) => &[(N(-1), N(5)), (Min, N(-1))],
            (true, Self::Not) => &[(N(0), N(0)), (N(-1), N(0)), (Max, N(0))],
            (true, Self::Compare) => &[(N(-1), N(0)), (N(0), N(-1)), (N(-3), N(-3)), (Min, Max)],
            (false, Self::Add) => &[(N(1), N(2)), (Max, N(0)), (Max, N(1)), (Max, Max)],
            (false, Self::Sub) => &[(N(5), N(3)), (Max, Max), (N(0), N(1)), (N(3), N(5))],
            (false, Self::Mul) => &[
                (N(6), N(7)),
                (Max, N(1)),
                (N(0), Max),
                (Max, N(2)),
                (Half, Half),
            ],
            (false, Self::Div) => &[(N(7), N(2)), (Max, N(1)), (Max, Max), (N(5), N(0))],
            (false, Self::Mod) => &[(N(7), N(3)), (Max, N(10)), (N(3), N(7)), (N(5), N(0))],
            (false, Self::And) => &[(Max, N(15)), (N(0xf0), N(0x3c))],
            (false, Self::Or) => &[(N(0xf0), N(0x0f)), (BeforeMax, N(1))],
            (false, Self::Xor) => &[(Max, N(1)), (N(0xff), N(0x0f))],
            (false, Self::Not) => &[(N(0), N(0)), (Max, N(0)), (N(0x0f), N(0))],
            (false, Self::Compare) => &[(N(1), N(2)), (N(2), N(1)), (Max, Max), (N(0), Max)],
            (_, Self::Bounds) => &[(Min, Max)],
        }
    }
}

struct IntegerType {
    signed: bool,
    bits: usize,
    min: BigInt,
    max: BigInt,
}

impl IntegerType {
    fn operand(&self, operand: Operand) -> BigInt {
        match operand {
            Operand::Constant(n) => n.into(),
            Operand::Min => self.min.clone(),
            Operand::Max => self.max.clone(),
            Operand::BeforeMax => &self.max - 1,
            Operand::Half => BigInt::from(1) << (self.bits / 2),
        }
    }

    fn expected(&self, operation: Operation, a: &BigInt, b: &BigInt) -> String {
        if matches!(operation, Operation::Div | Operation::Mod) && b == &BigInt::from(0) {
            return panic(0x12);
        }
        let value = match operation {
            Operation::Add => a + b,
            Operation::Sub => a - b,
            Operation::Mul => a * b,
            // BigInt division truncates toward zero; remainder has the dividend's sign.
            Operation::Div => a / b,
            Operation::Mod => a % b,
            Operation::And => a & b,
            Operation::Or => a | b,
            Operation::Xor => a ^ b,
            Operation::Not if self.signed => !a,
            Operation::Not => a ^ &self.max,
            Operation::Compare => {
                let comparisons = [a < b, a > b, a <= b, a >= b, a == b, a != b];
                comparisons
                    .iter()
                    .enumerate()
                    .fold(0u8, |mask, (bit, &set)| mask | (u8::from(set) << bit))
                    .into()
            }
            Operation::Bounds => {
                return format!("({}, {})", literal(&self.min), literal(&self.max));
            }
        };
        if matches!(
            operation,
            Operation::Add | Operation::Sub | Operation::Mul | Operation::Div
        ) && (value < self.min || value > self.max)
        {
            panic(0x11)
        } else {
            literal(&value)
        }
    }

    fn source(&self, name: &str) -> String {
        let family = if self.signed { "signed" } else { "unsigned" };
        let suffix = format!("{}{}", if self.signed { "I" } else { "U" }, self.bits);
        let mut source = format!(
            "import * from std;\nimport * from std.{family};\nimport * from std.dispatch;\ncontract IntegerOperations {{\n"
        );
        for operation in OPERATIONS {
            for &(a, b) in operation.inputs(self.signed) {
                let inputs = [self.operand(a), self.operand(b)];
                let args = inputs[..operation.arity()]
                    .iter()
                    .map(literal)
                    .collect::<Vec<_>>()
                    .join(", ");
                let expected = self.expected(operation, &inputs[0], &inputs[1]);
                writeln!(source, "// #[({args}) -> {expected}]").unwrap();
            }
            let parameters = ["a: uint256", "b: uint256"][..operation.arity()].join(", ");
            let returns = if matches!(operation, Operation::Bounds) {
                "uint256, uint256"
            } else {
                "uint256"
            };
            writeln!(
                source,
                "function {}{suffix}({parameters}) public returns ({returns}) {{",
                operation.name()
            )
            .unwrap();
            for (&variable, &parameter) in ["x", "y"]
                .iter()
                .zip(["a", "b"].iter())
                .take(operation.arity())
            {
                writeln!(
                    source,
                    "let {variable}: {name} = Typedef.abs(Typedef.rep({parameter}));"
                )
                .unwrap();
            }
            match operation {
                Operation::Compare => {
                    source.push_str("let r: uint256 = uint256(0);\n");
                    for (bit, comparison) in ["<", ">", "<=", ">=", "==", "!="].iter().enumerate() {
                        writeln!(
                            source,
                            "if (x {comparison} y) {{ r = r + uint256({}); }}",
                            1 << bit
                        )
                        .unwrap();
                    }
                    source.push_str("return r;\n");
                }
                Operation::Bounds => {
                    writeln!(
                        source,
                        "let lo: {name} = Bounded.minVal();\nlet hi: {name} = Bounded.maxVal();"
                    )
                    .unwrap();
                    source
                        .push_str("return (uint256(Typedef.rep(lo)), uint256(Typedef.rep(hi)));\n");
                }
                _ => writeln!(
                    source,
                    "return uint256(Typedef.rep({}));",
                    operation.expression()
                )
                .unwrap(),
            }
            source.push_str("}\n");
        }
        source.push_str("}\n");
        source
    }
}

fn panic(code: u8) -> String {
    format!("revert(0x4e487b71{code:064x})")
}

fn literal(value: &BigInt) -> String {
    let word = word(value);
    if word.bits() <= 64 {
        word.to_str_radix(10)
    } else {
        format!("0x{}", word.to_str_radix(16))
    }
}

fn word(value: &BigInt) -> BigInt {
    let modulus = BigInt::from(1) << 256usize;
    ((value % &modulus) + &modulus) % &modulus
}

/// Returns one source fixture per integer type (11 types, 433 execution cases).
/// Smaller contracts keep each dispatcher within the default evaluation fuel.
fn fixtures() -> Vec<(String, String)> {
    integer_types()
        .into_iter()
        .map(|(name, ty)| {
            let source = ty.source(&name);
            (name, source)
        })
        .collect()
}

fn integer_types() -> Vec<(String, IntegerType)> {
    let mut types = Vec::new();
    for signed in [true, false] {
        for bits in [8, 16, 32, 64, 128, 256] {
            if !signed && bits == 256 {
                continue; // uint256 already has separate checked-arithmetic coverage.
            }
            let magnitude = BigInt::from(1) << (bits - usize::from(signed));
            let ty = IntegerType {
                signed,
                bits,
                min: if signed { -&magnitude } else { BigInt::from(0) },
                max: magnitude - 1,
            };
            let name = format!("{}int{bits}", if signed { "" } else { "u" });
            types.push((name, ty));
        }
    }
    types
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::format::{FormatOptions, format_source};

    #[test]
    fn matrix_sources_parse_and_format() {
        let fixtures = fixtures();
        assert_eq!(fixtures.len(), 11);
        let mut cases = 0;
        for (name, source) in fixtures {
            assert_eq!(source.matches("function ").count(), 11, "{name}");
            cases += source.matches("// #[").count();
            let formatted = format_source(&source, &FormatOptions::default())
                .unwrap_or_else(|error| panic!("{name}: {error:?}"));
            assert_eq!(
                format_source(&formatted, &FormatOptions::default()).unwrap(),
                formatted,
                "{name}: formatting must be idempotent"
            );
        }
        assert_eq!(cases, 433);
    }

    #[test]
    fn abi_matrix_sources_parse_and_format() {
        let fixtures = abi::fixtures();
        assert_eq!(fixtures.len(), 11);
        let mut cases = 0;
        for (name, source) in fixtures {
            cases += source.matches("// #[raw(").count();
            let formatted = format_source(&source, &FormatOptions::default())
                .unwrap_or_else(|error| panic!("{name}: {error:?}"));
            assert_eq!(
                format_source(&formatted, &FormatOptions::default()).unwrap(),
                formatted,
                "{name}: formatting must be idempotent"
            );
        }
        assert_eq!(cases, 422);
    }
}
