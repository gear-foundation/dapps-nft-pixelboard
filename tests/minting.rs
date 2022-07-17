use ft_io::{ InitConfig as InitFT};
use gstd::ActorId;
use gtest::{Program, System};
use nft_io::InitNFT;

pub mod utils;
use utils::*;

#[test]
fn minting_failures() {
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

    let pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(
            FOREIGN_USER,
            InitNFTPixelboard {
                ft_program: ft_program.id().as_ref().try_into().unwrap(),
                min_block_side_length: 10,
                nft_program: nft_program.id().as_ref().try_into().unwrap(),
                owner: OWNER.into(),
                painting: vec![0; 100],
                pixel_price: MAX_PIXEL_PRICE,
                resale_commission_percentage: 100,
                resolution: (10, 10).into(),
            }
        )
        .log()
        .is_empty());

    let pixelboard_mint_action = NFTPixelboardAction::Mint {
        painting: vec![0; 25],
        rectangle: ((3, 3), (8, 8)).into(),
        token_metadata: Default::default(),
    };

    let mut failed_pixelboard_mint_action = pixelboard_mint_action.clone();
    if let NFTPixelboardAction::Mint {
        ref mut rectangle, ..
    } = failed_pixelboard_mint_action
    {
        *rectangle = ((8, 3), (3, 8)).into();
    } else {
        unreachable!();
    }
    assert!(pixelboard_program
        .send(FOREIGN_USER, failed_pixelboard_mint_action)
        .main_failed());

    failed_pixelboard_mint_action = pixelboard_mint_action.clone();
    if let NFTPixelboardAction::Mint {
        ref mut rectangle, ..
    } = failed_pixelboard_mint_action
    {
        *rectangle = ((8, 8), (3, 3)).into();
    } else {
        unreachable!();
    }
    assert!(pixelboard_program
        .send(FOREIGN_USER, failed_pixelboard_mint_action)
        .main_failed());

    failed_pixelboard_mint_action = pixelboard_mint_action.clone();
    if let NFTPixelboardAction::Mint {
        ref mut rectangle, ..
    } = failed_pixelboard_mint_action
    {
        *rectangle = ((3, 8), (8, 3)).into();
    } else {
        unreachable!();
    }
    assert!(pixelboard_program
        .send(FOREIGN_USER, failed_pixelboard_mint_action)
        .main_failed());

    failed_pixelboard_mint_action = pixelboard_mint_action.clone();
    if let NFTPixelboardAction::Mint {
        ref mut rectangle, ..
    } = failed_pixelboard_mint_action
    {
        rectangle.lower_right_corner = (11, 11).into();
    } else {
        unreachable!();
    }
    assert!(pixelboard_program
        .send(FOREIGN_USER, failed_pixelboard_mint_action)
        .main_failed());

    failed_pixelboard_mint_action = pixelboard_mint_action;
    if let NFTPixelboardAction::Mint {
        ref mut rectangle, ..
    } = failed_pixelboard_mint_action
    {
        rectangle.upper_left_corner = (11, 11).into();
    } else {
        unreachable!();
    }
    assert!(pixelboard_program
        .send(FOREIGN_USER, failed_pixelboard_mint_action)
        .main_failed());
}

#[test]
fn asdqwe() {
    let system = System::new();
    system.init_logger();

    let pixelboard_program = Program::current(&system);
    println!("{:?}", pixelboard_program
        .send(
            FOREIGN_USER,
            InitNFTPixelboard {
                ft_program: ActorId::zero(),
                min_block_side_length: 10,
                nft_program: ActorId::zero(),
                owner: OWNER.into(),
                painting: vec![0; 100],
                pixel_price: MAX_PIXEL_PRICE,
                resale_commission_percentage: 100,
                resolution: (10, 10).into(),
            }
        )
        .log()[0].payload());
}
