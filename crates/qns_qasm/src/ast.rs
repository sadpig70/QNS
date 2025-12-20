//! Abstract Syntax Tree for OpenQASM.

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub version: Option<String>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    QRegDecl {
        name: String,
        size: usize,
    },
    CRegDecl {
        name: String,
        size: usize,
    },
    GateDecl {
        name: String,
        params: Vec<String>,
        args: Vec<String>,
        body: Vec<GateOperation>,
    },
    GateCall {
        name: String,
        params: Vec<f64>,
        args: Vec<Argument>,
    },
    Measure {
        qubit: Argument,
        target: Argument,
    },
    Reset {
        qubit: Argument,
    },
    Barrier {
        args: Vec<Argument>,
    },
    If {
        condition: String,
        val: usize,
        body: Box<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateOperation {
    pub name: String,
    pub params: Vec<String>, // Parameters in definition can be expressions, simplified here
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Id(String),
    Indexed(String, usize),
}
