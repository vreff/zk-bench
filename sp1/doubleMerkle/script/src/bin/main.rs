use clap::Parser;
use double_merkle_lib::{
    build_tree, get_siblings, DoubleMerkleProofInput, DEPTH, LEAVES_A, LEAVES_B, PROVE_INDEX,
};
use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, ProvingKey, SP1Stdin,
};

const DOUBLE_MERKLE_ELF: Elf = include_elf!("double-merkle-program");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,
}

fn main() {
    sp1_sdk::utils::setup_logger();

    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    let levels_a = build_tree(&LEAVES_A);
    let root_a = levels_a[DEPTH][0];
    let siblings_a = get_siblings(&levels_a, PROVE_INDEX);

    let levels_b = build_tree(&LEAVES_B);
    let root_b = levels_b[DEPTH][0];
    let siblings_b = get_siblings(&levels_b, PROVE_INDEX);

    println!("Double Merkle Tree Membership Proof (SP1 zkVM)");
    println!("===============================================");
    println!("Tree A leaves: {:?}", LEAVES_A);
    println!("Tree B leaves: {:?}", LEAVES_B);
    println!("Proving index {} in both trees", PROVE_INDEX);
    println!();

    let input = DoubleMerkleProofInput {
        leaf_value_a: LEAVES_A[PROVE_INDEX as usize],
        leaf_index_a: PROVE_INDEX,
        siblings_a,
        leaf_value_b: LEAVES_B[PROVE_INDEX as usize],
        leaf_index_b: PROVE_INDEX,
        siblings_b,
    };

    let mut stdin = SP1Stdin::new();
    stdin.write(&input);

    let client = ProverClient::from_env();

    if args.execute {
        let (output, report) = client.execute(DOUBLE_MERKLE_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        let output_bytes: &[u8] = output.as_slice();
        let committed_root_a: [u8; 32] = output_bytes[..32].try_into().unwrap();
        let committed_root_b: [u8; 32] = output_bytes[32..64].try_into().unwrap();
        assert_eq!(committed_root_a, root_a, "Root A mismatch!");
        assert_eq!(committed_root_b, root_b, "Root B mismatch!");
        println!("Both roots match!");
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        println!("Generating proof...");
        let pk = client.setup(DOUBLE_MERKLE_ELF).expect("failed to setup elf");

        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        client
            .verify(&proof, pk.verifying_key(), None)
            .expect("failed to verify proof");
        println!("Successfully verified proof!");

        let proof_bytes = bincode::serialize(&proof).unwrap();
        std::fs::write("proof.bin", &proof_bytes).unwrap();
        println!("Proof saved to proof.bin ({} bytes)", proof_bytes.len());
    }
}
