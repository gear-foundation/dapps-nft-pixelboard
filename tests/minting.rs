pub mod utils;
use utils::*;

#[test]
fn minting_failures() {
    let system = initialize_system();

    let ft_program = FungibleToken::initialize(&system);
    ft_program.mint(FOREIGN_USER, MAX_PIXEL_PRICE * 25);

    let nft_program = NonFungibleToken::initialize(&system);

    let pixelboard_program =
        NFTPixelboard::initialize(&system, ft_program.actor_id(), nft_program.actor_id()).succeed();

    let default_painting = vec![0; 25];
    let default_rectangle = ((3, 3), (8, 8)).into();

    // Should fail because the coordinates are mixed up.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((8, 3), (3, 8)).into(),
        )
        .failed();
    // Should fail because the coordinates are mixed up.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((8, 8), (3, 3)).into(),
        )
        .failed();
    // Should fail because the coordinates are mixed up.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((3, 8), (8, 3)).into(),
        )
        .failed();
    // Should fail because the coordinates are mixed up.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((3, 3), (11, 11)).into(),
        )
        .failed();
    // Should fail because the coordinates are mixed up.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((11, 11), (8, 8)).into(),
        )
        .failed();
    // Should fail because pixel count in a painting must equal the count in a NFT.
    pixelboard_program
        .mint(FOREIGN_USER, vec![0; 24], default_rectangle)
        .failed();
    // Should fail because pixel count in a painting must equal the count in a NFT.
    pixelboard_program
        .mint(FOREIGN_USER, vec![0; 26], default_rectangle)
        .failed();

    pixelboard_program
        .mint(FOREIGN_USER, default_painting.clone(), default_rectangle)
        .check(0);

    // Should fail because the given NFT rectangle collides with an existing one.
    pixelboard_program
        .mint(FOREIGN_USER, default_painting.clone(), default_rectangle)
        .failed();
    // Should fail because the given NFT rectangle collides with an existing one.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((0, 0), (5, 5)).into(),
        )
        .failed();
    // Should fail because the given NFT rectangle collides with an existing one.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((5, 0), (10, 5)).into(),
        )
        .failed();
    // Should fail because the given NFT rectangle collides with an existing one.
    pixelboard_program
        .mint(
            FOREIGN_USER,
            default_painting.clone(),
            ((0, 5), (5, 10)).into(),
        )
        .failed();
    // Should fail because the given NFT rectangle collides with an existing one.
    pixelboard_program
        .mint(FOREIGN_USER, default_painting, ((5, 5), (10, 10)).into())
        .failed();
}
