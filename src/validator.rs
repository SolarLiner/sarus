use std::fmt::Display;

use crate::frontend::{Declaration, Expr};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum TypeError {
    #[error("Type mismatch; expected {expected}, found {actual}")]
    TypeMismatch { expected: Type, actual: Type },
    #[error("Tuple length mismatch; expected {expected} found {actual}")]
    TupleLengthMismatch { expected: usize, actual: usize },
    #[error("Function \"{0}\" does not exist")]
    UnknownFunction(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Void,
    Bool,
    Float,
    Tuple(Vec<Type>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "bool"),
            Type::Float => write!(f, "float"),
            Type::Tuple(inner) => {
                write!(f, "(")?;
                inner
                    .iter()
                    .map(|t| write!(f, "{}, ", t))
                    .collect::<Result<Vec<_>, _>>()?;
                write!(f, ")")
            }
        }
    }
}

impl Type {
    fn of(expr: &Expr, env: &[Declaration]) -> Result<Type, TypeError> {
        let res = match expr {
            Expr::Literal(_) | Expr::Identifier(_) => Type::Float,
            Expr::Binop(_, l, r) => {
                let lt = Type::of(l, env)?;
                let rt = Type::of(r, env)?;
                if lt == rt {
                    lt
                } else {
                    return Err(TypeError::TypeMismatch {
                        expected: lt,
                        actual: rt,
                    });
                }
            }
            Expr::Compare(_, _, _) => Type::Bool,
            Expr::IfThen(econd, _) => {
                let tcond = Type::of(econd, env)?;
                if tcond != Type::Bool {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Bool,
                        actual: tcond,
                    });
                }
                Type::Void
            }
            Expr::IfElse(econd, etrue, efalse) => {
                let tcond = Type::of(econd, env)?;
                if tcond != Type::Bool {
                    return Err(TypeError::TypeMismatch {
                        expected: Type::Bool,
                        actual: tcond,
                    });
                }

                let ttrue = etrue
                    .iter()
                    .map(|e| Type::of(e, env))
                    .collect::<Result<Vec<_>, _>>()?
                    .last()
                    .cloned()
                    .unwrap_or(Type::Void);
                let tfalse = efalse
                    .iter()
                    .map(|e| Type::of(e, env))
                    .collect::<Result<Vec<_>, _>>()?
                    .last()
                    .cloned()
                    .unwrap_or(Type::Void);

                if ttrue == tfalse {
                    ttrue
                } else {
                    return Err(TypeError::TypeMismatch {
                        expected: ttrue,
                        actual: tfalse,
                    });
                }
            }
            Expr::Assign(vars, e) => {
                let tlen = match e.len().into() {
                    1 => Type::of(&e[0], env)?.tuple_size(),
                    n => n,
                };
                if usize::from(vars.len()) != tlen {
                    return Err(TypeError::TupleLengthMismatch {
                        actual: vars.len().into(),
                        expected: e.len().into(),
                    });
                }
                Type::Tuple(
                    e.iter()
                        .map(|e| Type::of(e, env))
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            Expr::AssignOp(_, _, e) => Type::of(e, env)?,
            Expr::WhileLoop(_, _) => Type::Void,
            Expr::Block(b) => b
                .iter()
                .map(|e| Type::of(e, env))
                .last()
                .map(Result::unwrap)
                .unwrap_or(Type::Void),
            Expr::Call(fn_name, args) => {
                if let Some(d) = env.iter().filter(|d| &d.name == fn_name).next() {
                    if d.params.len() == args.len() {
                        let targs: Result<Vec<_>, _> =
                            args.iter().map(|e| Type::of(e, env)).collect();
                        match targs {
                            Ok(_) => match &d.returns {
                                v if v.is_empty() => Type::Void,
                                v if v.len() == 1 => Type::Float,
                                v => Type::Tuple(vec![Type::Float; v.len()]),
                            },
                            Err(err) => return Err(err),
                        }
                    } else {
                        return Err(TypeError::TupleLengthMismatch {
                            expected: d.params.len(),
                            actual: args.len(),
                        });
                    }
                } else {
                    return Err(TypeError::UnknownFunction(fn_name.to_string()));
                }
            }
            Expr::GlobalDataAddr(_) => Type::Float,
            Expr::Bool(_) => Type::Bool,
            Expr::Parentheses(expr) => Type::of(expr, env)?,
        };
        Ok(res)
    }

    pub fn tuple_size(&self) -> usize {
        match self {
            Type::Void => 0,
            Type::Bool | Type::Float => 1,
            Type::Tuple(v) => v.len(),
        }
    }
}

pub fn validate_program(decls: Vec<Declaration>) -> Result<Vec<Declaration>, TypeError> {
    for d in &decls {
        for expr in &d.body {
            Type::of(expr, &decls)?;
        }
    }
    Ok(decls)
}