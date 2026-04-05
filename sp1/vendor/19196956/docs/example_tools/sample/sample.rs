use std::env;
use zkp_ecc_lib::{from_kmx, analyze_ops};
use zkp_ecc_lib::Simulator;
use alloy_primitives::U256;
use sha3::{Shake256, digest::{Update, ExtendableOutput}};
use rand::Rng;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: sample <circuit_path> <register0_initial_value> ...");
        return;
    }
    let ops = from_kmx(&args[1]).unwrap();

    // Randomly seed the CSPRNG.
    let mut hasher = Shake256::default();
    let seed: [u8; 32] = rand::thread_rng().gen();
    hasher.update(&seed);
    let mut xof = hasher.finalize_xof();
    
    let (num_qubits, num_bits, _r, reg) = analyze_ops(ops.iter().copied());
    let mut sim = Simulator::new(
        num_qubits as usize,
        num_bits as usize,
        &mut xof,
    );

    if args.len() != reg.len() + 2 {
        eprintln!("The given circuit declares {} registers, but you passed {} initial value arguments.\nUsage: <circuit_path> <register0_initial_value> ...", reg.len(), args.len() - 2);
        return;
    }
    for k in 0..reg.len() {
        let v = args[k + 2].parse::<U256>().expect("Argument is not an integer");
        sim.set_register(&reg[k], v, 0);
    }

    sim.apply(&ops);

    for k in 0..reg.len() {
        if k > 0 {
            print!(" ");
        }
        print!("{}", sim.get_register(&reg[k], 0));
    }
    print!("\n");
}
