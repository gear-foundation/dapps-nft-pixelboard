pub mod utils;
use utils::*;

#[test]
fn reselling() {
    let system = initialize_system();

    let ft_program = FungibleToken::initialize(&system);
    ft_program.mint(USER[0], MAX_PIXEL_PRICE * 25);
    ft_program.mint(USER[1], MAX_PIXEL_PRICE * 25);

    let nft_program = NonFungibleToken::initialize(&system);

    let pixelboard_config = InitNFTPixelboard {
        ft_program: ft_program.actor_id(),
        block_side_length: 1,
        nft_program: nft_program.actor_id(),
        owner: OWNER.into(),
        painting: vec![0; 100],
        pixel_price: MAX_PIXEL_PRICE,
        resale_commission_percentage: 13,
        resolution: (10, 10).into(),
    };
    let pixelboard_program =
        NFTPixelboard::initialize_custom(&system, pixelboard_config.clone()).succeed();

    let mut token = Token(
        ((3, 3), (8, 8)).into(),
        TokenInfo {
            owner: USER[0].into(),
            token_id: 0.into(),
            pixel_price: Some(MAX_PIXEL_PRICE),
        },
    );

    pixelboard_program
        .mint(USER[0], vec![0; 25], token.0)
        .check(0);
    pixelboard_program
        .put_up_for_sale(USER[0], 0, MAX_PIXEL_PRICE)
        .check(0);

    pixelboard_program.meta_state().token_info(0).check(token);
    nft_program
        .meta_state()
        .owner(0)
        .check(pixelboard_program.actor_id());

    pixelboard_program.buy(USER[1], 0).check(0);
    token.1.owner = USER[1].into();
    token.1.pixel_price = None;

    let commision =
        MAX_PIXEL_PRICE * 25 * pixelboard_config.resale_commission_percentage as u128 / 100;
    ft_program
        .balance(OWNER)
        .check(MAX_PIXEL_PRICE * 25 + commision);
    ft_program
        .balance(USER[0])
        .check(MAX_PIXEL_PRICE * 25 - commision);
    ft_program.balance(USER[1]).check(0);
    nft_program.meta_state().owner(0).check(USER[1].into());
    pixelboard_program.meta_state().token_info(0).check(token);
}

#[test]
fn reselling_failures() {
    let system = initialize_system();

    let ft_program = FungibleToken::initialize(&system);
    ft_program.mint(USER[0], MAX_PIXEL_PRICE * 25);
    ft_program.mint(USER[1], MAX_PIXEL_PRICE * 24);

    let nft_program = NonFungibleToken::initialize(&system);

    let pixelboard_config = InitNFTPixelboard {
        ft_program: ft_program.actor_id(),
        block_side_length: 1,
        nft_program: nft_program.actor_id(),
        owner: OWNER.into(),
        painting: vec![0; 100],
        pixel_price: MAX_PIXEL_PRICE,
        resale_commission_percentage: 13,
        resolution: (10, 10).into(),
    };
    let pixelboard_program =
        NFTPixelboard::initialize_custom(&system, pixelboard_config.clone()).succeed();

    pixelboard_program
        .mint(USER[0], vec![0; 25], ((3, 3), (8, 8)).into())
        .check(0);
    // Should fail because FOREIGN_USER isn't the owner of the token.
    pixelboard_program
        .put_up_for_sale(FOREIGN_USER, 0, MAX_PIXEL_PRICE)
        .failed();
    // Should fail because `pixel_price` mustn't be more than `MAX_PIXEL_PRICE`.
    pixelboard_program
        .put_up_for_sale(USER[0], 0, MAX_PIXEL_PRICE + 1)
        .failed();
    // Should fail because the NFT isn't for sale.
    pixelboard_program.buy(USER[1], 0).failed();

    pixelboard_program
        .put_up_for_sale(USER[0], 0, MAX_PIXEL_PRICE)
        .check(0);

    // Should fail because the NFT is already for sale.
    pixelboard_program
        .put_up_for_sale(USER[0], 0, MAX_PIXEL_PRICE)
        .failed();
    // Should fail because USER[0] doesn't have enough tokens to buy this NFT.
    pixelboard_program.buy(USER[1], 0).failed();

    // But a commission should still be debited from USER[0] because USER[0] has enough tokens for it.
    let commision =
        MAX_PIXEL_PRICE * 25 * pixelboard_config.resale_commission_percentage as u128 / 100;
    ft_program
        .balance(USER[1])
        .check(MAX_PIXEL_PRICE * 24 - commision);
    ft_program
        .balance(OWNER)
        .check(MAX_PIXEL_PRICE * 25 + commision);
}
