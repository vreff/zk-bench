use clap::Parser;
use merkle_lib::{build_tree, get_siblings, MerkleProofInput, DEPTH, LEAVES, PROVE_INDEX};
use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, ProvingKey, SP1Stdin,
};

const MERKLE_ELF: Elf = include_elf!("merkle-program");

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

    // Build the Merkle tree on the host side
    let levels = build_tree(&LEAVES);
    let expected_root = levels[DEPTH][0];
    let siblings = get_siblings(&levels, PROVE_INDEX);

    println!("Merkle Tree Membership Proof (SP1 zkVM)");
    println!("=======================================");
    println!("Leaves: {:?}", LEAVES);
    println!(
        "Proving membership of value {} at index {}",
        LEAVES[PROVE_INDEX as usize], PROVE_INDEX
    );
    println!("Root: 0x{}", hex::encode(expected_root));
    println!();

    // Prepare inputs for the guest program
    let input = MerkleProofInput {
        leaf_value: LEAVES[PROVE_INDEX as usize],
        leaf_index: PROVE_INDEX,
        siblings,
    };

    let mut stdin = SP1Stdin::new();
    stdin.write(&input);

    let client = ProverClient::from_env();

    if args.execute {
        let (output, report) = client.execute(MERKLE_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        let committed_root: [u8; 32] = output.as_slice().try_into().unwrap();
        println!("Committed root: 0x{}", hex::encode(committed_root));
        assert_eq!(committed_root, expected_root, "Root mismatch!");
        println!("Root matches expected value!");
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        println!("Generating proof...");
        let pk = client.setup(MERKLE_ELF).expect("failed to setup elf");

        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        client
            .verify(&proof, pk.verifying_key(), None)
            .expect("failed to verify proof");
        println!("Successfully verified proof!");

        // Save proof
        let proof_bytes = bincode::serialize(&proof).unwrap();
        std::fs::write("proof.bin", &proof_bytes).unwrap();
        println!();
        println!("Proof saved to proof.bin ({} bytes)", proof_bytes.len());
        println!();
        println!("Public outputs: root hash");
        println!(
            "Private inputs: leaf value ({}), index ({}), siblings",
            LEAVES[PROVE_INDEX as usize], PROVE_INDEX
        );
    }
}
