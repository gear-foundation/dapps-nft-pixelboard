use ft_io::InitConfig as InitFT;
use gstd::ActorId;
use gtest::{Program, System};
use nft_io::InitNFT;
use nft_pixelboard_io::*;

mod utils;
use utils::*;

#[test]
fn initialization() {
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

    let pixelboard_config = InitNFTPixelboard {
        ft_program: ft_program.id().as_ref().try_into().unwrap(),
        min_block_side_length: 10,
        nft_program: nft_program.id().as_ref().try_into().unwrap(),
        owner: FOREIGN_USER.into(),
        painting: vec![0; 100],
        pixel_price: MAX_PIXEL_PRICE,
        resale_commission_percentage: 100,
        resolution: (10, 10).into(),
    };

    let mut custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.ft_program = ActorId::zero();
    let mut pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.nft_program = ActorId::zero();
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.min_block_side_length = 0;
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.resolution.width = 0;
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.resolution.height = 0;
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.resolution = (0, 0).into();
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.resolution.width = 15;
    custom_pixelboard_config.painting = vec![1; 150];
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.resolution.height = 15;
    custom_pixelboard_config.painting = vec![1; 150];
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.resolution = (15, 15).into();
    custom_pixelboard_config.painting = vec![1; 225];
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.resale_commission_percentage = 101;
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.pixel_price = MAX_PIXEL_PRICE + 1;
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.painting = vec![1; 101];
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    custom_pixelboard_config = pixelboard_config.clone();
    custom_pixelboard_config.painting = vec![1; 99];
    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, custom_pixelboard_config)
        .main_failed());

    pixelboard_program = Program::current(&system);
    assert!(pixelboard_program
        .send(FOREIGN_USER, pixelboard_config.clone())
        .log()
        .is_empty());

    if let NFTPixelboardStateReply::FTProgram(reply) = pixelboard_program
        .meta_state(NFTPixelboardState::FTProgram)
        .unwrap()
    {
        assert_eq!(reply, ft_program.id().as_ref().try_into().unwrap());
    } else {
        unreachable!();
    }

    if let NFTPixelboardStateReply::NFTProgram(reply) = pixelboard_program
        .meta_state(NFTPixelboardState::NFTProgram)
        .unwrap()
    {
        assert_eq!(reply, nft_program.id().as_ref().try_into().unwrap());
    } else {
        unreachable!();
    }

    if let NFTPixelboardStateReply::MinBlockSideLength(reply) = pixelboard_program
        .meta_state(NFTPixelboardState::MinBlockSideLength)
        .unwrap()
    {
        assert_eq!(reply, pixelboard_config.min_block_side_length);
    } else {
        unreachable!();
    }

    if let NFTPixelboardStateReply::Painting(reply) = pixelboard_program
        .meta_state(NFTPixelboardState::Painting)
        .unwrap()
    {
        assert_eq!(reply, pixelboard_config.painting);
    } else {
        unreachable!();
    }

    if let NFTPixelboardStateReply::PixelPrice(reply) = pixelboard_program
        .meta_state(NFTPixelboardState::PixelPrice)
        .unwrap()
    {
        assert_eq!(reply, pixelboard_config.pixel_price);
    } else {
        unreachable!();
    }

    if let NFTPixelboardStateReply::ResaleCommissionPercentage(reply) = pixelboard_program
        .meta_state(NFTPixelboardState::ResaleCommissionPercentage)
        .unwrap()
    {
        assert_eq!(reply, pixelboard_config.resale_commission_percentage);
    } else {
        unreachable!();
    }

    if let NFTPixelboardStateReply::Resolution(reply) = pixelboard_program
        .meta_state(NFTPixelboardState::Resolution)
        .unwrap()
    {
        assert_eq!(reply, pixelboard_config.resolution);
    } else {
        unreachable!();
    }
}
