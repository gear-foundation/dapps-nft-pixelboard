use ft_io::{FTAction, InitConfig as InitFT};
use gstd::prelude::*;
use gtest::{Program, System};
use nft_io::InitNFT;
use nft_pixelboard_io::*;

pub fn init_ft(sys: &System) -> Program {
    let ft_program = Program::from_file(sys, "./target/fungible_token.wasm");

    assert!(ft_program
        .send(
            123123,
            InitFT {
                name: String::from("MyToken"),
                symbol: String::from("MTK"),
            },
        )
        .log()
        .is_empty());

    ft_program
}

pub fn init_nft(system: &System) -> Program {
    let nft_program = Program::from_file(system, "./target/nft.wasm");

    assert!(nft_program
        .send(
            123123,
            InitNFT {
                name: "Item".into(),
                symbol: "ITM".into(),
                base_uri: Default::default(),
                royalties: Default::default(),
            },
        )
        .log()
        .is_empty());

    nft_program
}

#[test]
fn test() {
    let system = System::new();
    system.init_logger();
    let prog = Program::current(&system);

    let ft = init_ft(&system);

    ft.send(123, FTAction::Mint(10000));

    init_nft(&system);

    let mut f = prog.send(
        123123,
        InitNFTPixelboard {
            ft_program: 2.into(),
            min_block_side_length: 10,
            nft_program: 3.into(),
            owner: 123124.into(),
            painting: vec![0; 2500],
            pixel_price: 1,
            resale_commission_percentage: 0,
            resolution: (50, 50).into(),
        },
    );
    f.log().is_empty();

    let d = prog.send(
        123,
        NFTPixelboardAction::Mint {
            token_metadata: Default::default(),
            rectangle: ((20, 10), (30, 20)).into(),
            painting: vec![1; 100],
        },
    );

    dbg!(d.main_gas_burned());
    dbg!(d.others_gas_burned());

    assert!(d.contains(&(123, NFTPixelboardEvent::Minted(0.into()).encode())));

    assert!(prog
        .send(
            123,
            NFTPixelboardAction::Mint {
                token_metadata: Default::default(),
                rectangle: ((10, 20), (20, 30)).into(),
                painting: vec![7; 100]
            }
        )
        .contains(&(123, NFTPixelboardEvent::Minted(1.into()).encode())));

    let asdas: NFTPixelboardStateReply = prog.meta_state(NFTPixelboardState::Painting).unwrap();
    if let NFTPixelboardStateReply::Painting(pai) = asdas {
        for row in pai.chunks_exact(50) {
            println!("{row:?}");
        }
    } else {
        panic!("AAAAAA")
    }

    let asdas: NFTPixelboardStateReply = prog
        .meta_state(NFTPixelboardState::PixelInfo((10, 40).into()))
        .unwrap();
    if let NFTPixelboardStateReply::PixelInfo(pai) = asdas {
        println!("{:?}", pai);
    } else {
        panic!("AAAAAA")
    }

    let asdas: NFTPixelboardStateReply = prog
        .meta_state(NFTPixelboardState::TokenInfo(12.into()))
        .unwrap();
    if let NFTPixelboardStateReply::TokenInfo(pai) = asdas {
        println!("{:?}", pai);
    } else {
        panic!("AAAAAA")
    }
}
