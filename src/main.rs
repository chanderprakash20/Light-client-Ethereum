use web3::transports::Http;
use web3::Web3;
use web3::types::{Block, BlockId, BlockNumber, H256, Transaction};
use tokio;
use sha3::{Digest, Keccak256};
use hex;

#[tokio::main]
async fn main() {
    // Connect to an Ethereum node using Infura
    let transport = Http::new("https://mainnet.infura.io/v3/364a2d379e9949bf911c29a92183709d").unwrap();
    let web3 = Web3::new(transport);

    // Fetch the latest block number
    let latest_block_number = web3.eth().block_number().await.unwrap();
    println!("Latest block number: {:?}", latest_block_number);

    // Fetch the block data
    let block: Block<Transaction> = web3.eth().block_with_txs(BlockId::Number(BlockNumber::Number(latest_block_number))).await.unwrap().unwrap();
    println!("Block data: {:?}", block);

    // Example transaction hash
    let tx_hash = "0xf9647b01368cfe0dcbb0241aeff2416a50365398bf6c2e4b2b6ce180ec3ead43";
    let tx_hash_bytes = hex::decode(&tx_hash[2..]).unwrap();

    // Fetch the specific transaction
    let transaction = block.transactions.iter().find(|tx| tx.hash == H256::from_slice(&tx_hash_bytes)).unwrap();
    println!("Transaction: {:?}", transaction);

    // Calculate the Merkle proof
    let tx_index = block.transactions.iter().position(|tx| tx.hash == H256::from_slice(&tx_hash_bytes)).unwrap();
    let merkle_proof = calculate_merkle_proof(&block.transactions, tx_index);
    println!("Merkle proof: {:?}", merkle_proof);

    // Extract the state root from the block header
    let state_root = block.state_root;
    println!("State root: {:?}", state_root);

    // Verify the Merkle proof using the transaction hash and the state root from the block header
    let is_valid = verify_merkle_proof(merkle_proof, &tx_hash[2..], &state_root);
    println!("Is the Merkle proof valid? {}", is_valid);
}

fn calculate_merkle_proof(transactions: &Vec<Transaction>, mut tx_index: usize) -> Vec<(H256, bool)> {
    let mut proof = Vec::new();
    let mut hashes: Vec<H256> = transactions.iter().map(|tx| tx.hash).collect();

    while hashes.len() > 1 {
        if hashes.len() % 2 != 0 {
            hashes.push(hashes.last().unwrap().clone());
        }

        let mut new_hashes = Vec::new();
        for i in (0..hashes.len()).step_by(2) {
            let left = hashes[i];
            let right = hashes[i + 1];

            let mut hasher = Keccak256::new();
            hasher.update(left.as_bytes());
            hasher.update(right.as_bytes());
            let combined_hash = H256::from_slice(&hasher.finalize());

            new_hashes.push(combined_hash);

            if i / 2 == tx_index / 2 {
                proof.push(if tx_index % 2 == 0 { (right, true) } else { (left, false) });
            }
        }

        hashes = new_hashes;
        tx_index /= 2;
    }

    proof
}

fn verify_merkle_proof(proof: Vec<(H256, bool)>, tx_hash: &str, block_root_hash: &H256) -> bool {
    let tx_hash_bytes = hex::decode(tx_hash).unwrap();
    let mut hash = H256::from_slice(&tx_hash_bytes);

    for (sibling, is_right_sibling) in proof {
        let mut hasher = Keccak256::new();

        if is_right_sibling {
            hasher.update(hash.as_bytes());
            hasher.update(sibling.as_bytes());
        } else {
            hasher.update(sibling.as_bytes());
            hasher.update(hash.as_bytes());
        }

        hash = H256::from_slice(&hasher.finalize());
    }

    // Compare the calculated root hash with the block header's root hash
    hash == *block_root_hash
}
