use gtest::{System, Program};
use ft_io::InitConfig as InitFT;
use nft_io::InitNFT;

mod utils;
use utils::*;

#[test]
fn minting() {
    let system = System::new();
    system.init_logger();

    let ft_program = Program::from_file(&system, "./target/fungible_token.wasm");
    assert!(ft_program
        .send(
            FOREIGN_USER,
            InitFT {
                name: Default::default(),
                symbol: Default::default(),
            },
        )
        .log()
        .is_empty());

    let nft_program = Program::from_file(&system, "./target/nft.wasm");
    assert!(nft_program
        .send(
            FOREIGN_USER,
            InitNFT {
                base_uri: Default::default(),
                name: Default::default(),
                royalties: Default::default(),
                symbol: Default::default(),
            }
        )
        .log()
        .is_empty());
}
