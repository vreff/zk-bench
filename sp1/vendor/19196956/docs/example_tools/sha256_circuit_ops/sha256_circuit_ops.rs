use std::env;
use zkp_ecc_lib::from_kmx;
use sha2::Sha256;
use sha2::digest::Update;
use sha2::digest::FixedOutput;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: sample <circuit_path>");
        return;
    }

    let circuit_operations = from_kmx(&args[1])
        .unwrap_or_else(|_| panic!("Failed to load circuit from {}", args[1]));
    let circuit_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&circuit_operations).expect("Failed to serialize operations");

    let mut hasher = Sha256::default();
    hasher.update(&circuit_bytes);
    let circuit_hash: [u8; 32] = hasher.finalize_fixed().into();
    let circuit_hash_hex_string: String = circuit_hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!("{}", circuit_hash_hex_string);
}
