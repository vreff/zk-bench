/// This file contains code for working with kickmix circuit files.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum OperationType {
    Neg = 0,
    Register = 1,
    AppendToRegister = 2,
    BitInvert = 3,
    BitStore0 = 4,
    BitStore1 = 5,
    X = 6,
    Z = 7,
    CX = 8,
    CZ = 9,
    Swap = 10,
    R = 11,
    Hmr = 12,
    CCX = 13,
    CCZ = 14,
    PushCondition = 15,
    PopCondition = 16,
    DebugPrint = 17,
}

impl OperationType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "NEG" => Some(Self::Neg),
            "REGISTER" => Some(Self::Register),
            "APPEND_TO_REGISTER" => Some(Self::AppendToRegister),
            "BIT_INVERT" => Some(Self::BitInvert),
            "BIT_STORE0" => Some(Self::BitStore0),
            "BIT_STORE1" => Some(Self::BitStore1),
            "X" => Some(Self::X),
            "Z" => Some(Self::Z),
            "CX" => Some(Self::CX),
            "CZ" => Some(Self::CZ),
            "SWAP" => Some(Self::Swap),
            "R" => Some(Self::R),
            "HMR" => Some(Self::Hmr),
            "CCX" => Some(Self::CCX),
            "CCZ" => Some(Self::CCZ),
            "PUSH_CONDITION" => Some(Self::PushCondition),
            "POP_CONDITION" => Some(Self::PopCondition),
            "DEBUG_PRINT" => Some(Self::DebugPrint),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct QubitId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct BitId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct RegisterId(pub u32);

pub const NO_QUBIT: QubitId = QubitId(u32::MAX);
pub const NO_BIT: BitId = BitId(u32::MAX);
pub const NO_REG: RegisterId = RegisterId(u32::MAX);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum QubitOrBit {
    Qubit(QubitId),
    Bit(BitId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Op {
    pub kind: OperationType,
    pub q_control2: QubitId,
    pub q_control1: QubitId,
    pub q_target: QubitId,
    pub c_target: BitId,
    pub c_condition: BitId,
    pub r_target: RegisterId,
}

impl Op {
    pub fn empty() -> Self {
        Self {
            kind: OperationType::Neg,
            q_control2: NO_QUBIT,
            q_control1: NO_QUBIT,
            q_target: NO_QUBIT,
            c_target: NO_BIT,
            c_condition: NO_BIT,
            r_target: NO_REG,
        }
    }

    pub fn from_text(line: &str) -> Option<Self> {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() || words[0].starts_with('#') {
            return None;
        }

        let mut out = Self::empty();

        if let Some(kind) = OperationType::from_name(words[0]) {
            out.kind = kind;
        } else {
            panic!("Unrecognized operation type '{}'", words[0]);
        }

        let mut cur_word = 1;

        if cur_word < words.len() && words[cur_word].starts_with('q') {
            out.q_target.0 = words[cur_word][1..].parse().unwrap();
            cur_word += 1;

            if cur_word < words.len() && words[cur_word].starts_with('q') {
                out.q_control1 = out.q_target;
                out.q_target.0 = words[cur_word][1..].parse().unwrap();
                cur_word += 1;
            }

            if cur_word < words.len() && words[cur_word].starts_with('q') {
                out.q_control2 = out.q_control1;
                out.q_control1 = out.q_target;
                out.q_target.0 = words[cur_word][1..].parse().unwrap();
                cur_word += 1;
            }
        }

        if cur_word < words.len() && words[cur_word].starts_with('b') {
            out.c_target.0 = words[cur_word][1..].parse().unwrap();
            cur_word += 1;
        }
        if cur_word < words.len() && words[cur_word].starts_with('r') {
            out.r_target.0 = words[cur_word][1..].parse().unwrap();
            cur_word += 1;
        }
        if cur_word + 1 < words.len()
            && words[cur_word] == "if"
            && words[cur_word + 1].starts_with('b')
        {
            out.c_condition.0 = words[cur_word + 1][1..].parse().unwrap();
            cur_word += 2;
        }

        if cur_word != words.len() {
            panic!("Failed to parse line '{}'", line);
        }

        Some(out)
    }
}



pub fn analyze_ops(ops: impl Iterator<Item = Op>) -> (u32, u32, u32, Vec<Vec<QubitOrBit>>) {
    let mut registers: Vec<Vec<QubitOrBit>> = Vec::new();
    let mut num_qubits = 0;
    let mut num_bits = 0;
    let mut num_registers = 0;

    for native_op in ops {
        if native_op.q_control2 != NO_QUBIT {
            num_qubits = num_qubits.max(native_op.q_control2.0 + 1);
        }
        if native_op.q_control1 != NO_QUBIT {
            num_qubits = num_qubits.max(native_op.q_control1.0 + 1);
        }
        if native_op.q_target != NO_QUBIT {
            num_qubits = num_qubits.max(native_op.q_target.0 + 1);
        }
        if native_op.c_target != NO_BIT {
            num_bits = num_bits.max(native_op.c_target.0 + 1);
        }
        if native_op.c_condition != NO_BIT {
            num_bits = num_bits.max(native_op.c_condition.0 + 1);
        }
        if native_op.r_target != NO_REG {
            num_registers = num_registers.max(native_op.r_target.0 + 1);
            while registers.len() <= native_op.r_target.0 as usize {
                registers.push(Vec::new());
            }
        }
        if native_op.kind == OperationType::AppendToRegister {
            if native_op.q_target != NO_QUBIT {
                registers[native_op.r_target.0 as usize].push(QubitOrBit::Qubit(native_op.q_target));
            }
            if native_op.c_target != NO_BIT {
                registers[native_op.r_target.0 as usize].push(QubitOrBit::Bit(native_op.c_target));
            }
        }
    }
    
    (num_qubits, num_bits, num_registers, registers)
}

pub fn from_kmx<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Op>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut operations = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if let Some(op) = Op::from_text(&line) {
            operations.push(op);
        }
    }

    Ok(operations)
}
