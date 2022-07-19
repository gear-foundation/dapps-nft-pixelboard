use gstd::ActorId;

pub mod utils;
use utils::*;

#[test]
fn initialization_failures() {
    let system = initialize_system();

    let ft_program = FungibleToken::initialize(&system);
    let nft_program = NonFungibleToken::initialize(&system);

    let pixelboard_config = InitNFTPixelboard {
        ft_program: ft_program.actor_id(),
        min_block_side_length: 10,
        nft_program: nft_program.actor_id(),
        owner: FOREIGN_USER.into(),
        painting: vec![0; 100],
        pixel_price: MAX_PIXEL_PRICE,
        resale_commission_percentage: 100,
        resolution: (10, 10).into(),
    };

    let mut failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.owner = ActorId::zero();
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.ft_program = ActorId::zero();
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.nft_program = ActorId::zero();
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.min_block_side_length = 0;
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.resolution.width = 0;
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.resolution.height = 0;
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.resolution = (0, 0).into();
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.resolution.width = 15;
    failed_pixelboard_config.painting = vec![1; 150];
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.resolution.height = 15;
    failed_pixelboard_config.painting = vec![1; 150];
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.resolution = (15, 15).into();
    failed_pixelboard_config.painting = vec![1; 225];
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.resale_commission_percentage = 101;
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.pixel_price = MAX_PIXEL_PRICE + 1;
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config.clone();
    failed_pixelboard_config.painting = vec![1; 101];
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();

    failed_pixelboard_config = pixelboard_config;
    failed_pixelboard_config.painting = vec![1; 99];
    NFTPixelboard::initialize_custom(&system, failed_pixelboard_config).failed();
}

#[test]
fn initialization_n_meta_state() {
    let system = initialize_system();

    let ft_program = FungibleToken::initialize(&system);
    let nft_program = NonFungibleToken::initialize(&system);

    let pixelboard_config = InitNFTPixelboard {
        ft_program: ft_program.actor_id(),
        min_block_side_length: 10,
        nft_program: nft_program.actor_id(),
        owner: FOREIGN_USER.into(),
        painting: vec![0; 100],
        pixel_price: MAX_PIXEL_PRICE,
        resale_commission_percentage: 100,
        resolution: (10, 10).into(),
    };
    let pixelboard_program =
        NFTPixelboard::initialize_custom(&system, pixelboard_config.clone()).succeed();

    pixelboard_program
        .meta_state()
        .ft_program()
        .check(ft_program.actor_id());
    pixelboard_program
        .meta_state()
        .nft_program()
        .check(nft_program.actor_id());
    pixelboard_program
        .meta_state()
        .min_block_side_length()
        .check(pixelboard_config.min_block_side_length);
    pixelboard_program
        .meta_state()
        .painting()
        .check(pixelboard_config.painting);
    pixelboard_program
        .meta_state()
        .pixel_price()
        .check(pixelboard_config.pixel_price);
    pixelboard_program
        .meta_state()
        .resale_commission_percentage()
        .check(pixelboard_config.resale_commission_percentage);
    pixelboard_program
        .meta_state()
        .resolution()
        .check(pixelboard_config.resolution);
}
