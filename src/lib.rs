#![no_std]

use gear_lib::non_fungible_token::token::{TokenId, TokenMetadata};
use gstd::{async_main, exec, msg, prelude::*, ActorId};
use nft_pixelboard_io::*;

mod utils;
use utils::*;

fn get_pixel_count<P: Into<usize>>(width: P, height: P) -> usize {
    let pixel_count = width.into() * height.into();
    if pixel_count == 0 {
        panic!("Width or height of a canvas/NFT mustn't be 0");
    };
    pixel_count
}

fn check_painting(painting: &Vec<Color>, pixel_count: usize) {
    if painting.len() != pixel_count {
        panic!("Pixel count in a painting must equal the count in a canvas/NFT");
    }
}

fn check_pixel_price(pixel_price: u128) {
    if pixel_price > MAX_PIXEL_PRICE {
        panic!("Pixel price must't be more than 2^96");
    }
}

fn get_mut_token_by_id<'a>(
    rectangles: &'a BTreeMap<TokenId, Rectangle>,
    tokens: &'a mut BTreeMap<Rectangle, TokenInfo>,
    token_id: TokenId,
) -> (&'a Rectangle, &'a mut TokenInfo) {
    let rectangle = rectangles.get(&token_id).expect("NFT not found by the ID");
    (
        rectangle,
        tokens
            .get_mut(rectangle)
            .expect("NFT not found by the rectangle"),
    )
}

fn paint(
    canvas_resolution: Resolution,
    rectangle: &Rectangle,
    rectangle_width: usize,
    rectangle_height: usize,
    canvas_painting: &mut [Color],
    token_painting: Vec<Color>,
) {
    let canvas_width = canvas_resolution.width as usize;

    let first_row_end = canvas_width * rectangle.upper_left_corner.y as usize
        + rectangle.lower_right_corner.x as usize;
    let first_row_start = first_row_end - rectangle_width;

    let (first_row_painting, rest_of_painting) = token_painting.split_at(rectangle_width);
    canvas_painting[first_row_start..first_row_end].copy_from_slice(first_row_painting);

    for (canvas_row, painting_row) in canvas_painting
        [first_row_end..first_row_end + (rectangle_height - 1) * canvas_width]
        .chunks_exact_mut(canvas_resolution.width as _)
        .zip(rest_of_painting.chunks_exact(rectangle_width))
    {
        canvas_row[canvas_width - rectangle_width..].copy_from_slice(painting_row);
    }
}

#[derive(Default)]
struct NFTPixelboard {
    owner: ActorId,
    min_block_side_length: MinBlockSideLength,
    pixel_price: u128,
    resolution: Resolution,
    resale_commission_percentage: u8,
    painting: Vec<Color>,

    rectangles_by_token_ids: BTreeMap<TokenId, Rectangle>,
    tokens_by_rectangles: BTreeMap<Rectangle, TokenInfo>,

    ft_program: ActorId,
    nft_program: ActorId,
}

impl NFTPixelboard {
    async fn mint(
        &mut self,
        rectangle: Rectangle,
        token_metadata: TokenMetadata,
        painting: Vec<Color>,
    ) {
        // Coordinates checks

        if rectangle.upper_left_corner.x > rectangle.lower_right_corner.x
            || rectangle.upper_left_corner.y > rectangle.lower_right_corner.y
        {
            panic!("Coordinates are mixed up or belong to wrong corners");
        }

        if rectangle.lower_right_corner.x > self.resolution.width
            || rectangle.lower_right_corner.y > self.resolution.height
        {
            panic!("Coordinates are out of the canvas");
        }

        if self.tokens_by_rectangles.keys().any(|existing_rectangle| {
            existing_rectangle.upper_left_corner.x < rectangle.lower_right_corner.x
                && existing_rectangle.lower_right_corner.x > rectangle.upper_left_corner.x
                && existing_rectangle.upper_left_corner.y < rectangle.lower_right_corner.y
                && existing_rectangle.lower_right_corner.y > rectangle.upper_left_corner.y
        }) {
            panic!("Given NFT rectangle collides with an existing NFT rectangle");
        }

        // Payment and NFT minting

        let rectangle_width = rectangle.width() as usize;
        let rectangle_height = rectangle.height() as usize;
        let rectangle_pixel_count = get_pixel_count(rectangle_width, rectangle_height);

        transfer_ftokens(
            self.ft_program,
            msg::source(),
            self.owner,
            rectangle_pixel_count as u128 * self.pixel_price,
        )
        .await;

        let token_id = mint_nft(self.nft_program, token_metadata).await;
        add_nft_approval(self.nft_program, exec::program_id(), token_id).await;
        transfer_nft(self.nft_program, msg::source(), token_id).await;

        // Painting

        check_painting(&painting, rectangle_pixel_count);
        paint(
            self.resolution,
            &rectangle,
            rectangle_width,
            rectangle_height,
            &mut self.painting,
            painting,
        );

        // Insertion and replying

        self.tokens_by_rectangles.insert(
            rectangle,
            TokenInfo {
                owner: msg::source(),
                pixel_price: None,
                token_id,
            },
        );
        self.rectangles_by_token_ids.insert(token_id, rectangle);

        reply(NFTPixelboardEvent::Minted(token_id));
    }

    async fn buy(&mut self, token_id: TokenId) {
        let (rectangle, token) = get_mut_token_by_id(
            &self.rectangles_by_token_ids,
            &mut self.tokens_by_rectangles,
            token_id,
        );

        let pixel_price = token
            .pixel_price
            .unwrap_or_else(|| panic!("NFT isn't for sale"));
        transfer_ftokens(
            self.ft_program,
            msg::source(),
            token.owner,
            get_pixel_count(
                rectangle.lower_right_corner.x - rectangle.upper_left_corner.x,
                rectangle.lower_right_corner.y - rectangle.upper_left_corner.y,
            ) as u128
                * pixel_price,
        )
        .await;

        token.pixel_price = None;
        token.owner = msg::source();
        transfer_nft(self.nft_program, msg::source(), token_id).await;

        reply(NFTPixelboardEvent::Bought(token_id));
    }

    async fn put_up_for_sale(&mut self, token_id: TokenId, pixel_price: u128) {
        let token = get_mut_token_by_id(
            &self.rectangles_by_token_ids,
            &mut self.tokens_by_rectangles,
            token_id,
        )
        .1;
        assert_eq!(token.owner, msg::source());

        check_pixel_price(pixel_price);
        token.pixel_price = Some(pixel_price);

        reply(NFTPixelboardEvent::ForSale(token_id));
    }

    fn paint(&mut self, token_id: TokenId, painting: Vec<Color>) {
        let (rectangle, token) = get_mut_token_by_id(
            &self.rectangles_by_token_ids,
            &mut self.tokens_by_rectangles,
            token_id,
        );
        assert_eq!(token.owner, msg::source());

        let rectangle_width = rectangle.width() as usize;
        let rectangle_height = rectangle.height() as usize;
        check_painting(
            &painting,
            get_pixel_count(rectangle_width, rectangle_height),
        );

        paint(
            self.resolution,
            rectangle,
            rectangle_width,
            rectangle_height,
            &mut self.painting,
            painting,
        );

        reply(NFTPixelboardEvent::Painted(token_id));
    }
}

static mut PROGRAM: Option<NFTPixelboard> = None;

#[no_mangle]
extern "C" fn init() {
    let config: InitNFTPixelboard = msg::load().expect("Unable to decode `InitNFTPixelboard`");

    if config.ft_program == ActorId::zero() {
        panic!("FT program address can't be 0");
    }

    if config.nft_program == ActorId::zero() {
        panic!("NFT program address can't be 0");
    }

    if config.min_block_side_length == 0 {
        panic!("`min_block_side_length` must be greater than 0");
    }

    check_painting(
        &config.painting,
        get_pixel_count(config.resolution.width, config.resolution.height),
    );

    if config.resolution.width % config.min_block_side_length != 0
        || config.resolution.height % config.min_block_side_length != 0
    {
        panic!("Each side of `resolution` must be a multiple of `min_block_side_length`");
    }

    if config.resale_commission_percentage > 100 {
        panic!("`resale_commission_percentage` mustn't be more than 100");
    }

    check_pixel_price(config.pixel_price);

    let program = NFTPixelboard {
        owner: config.owner,
        ft_program: config.ft_program,
        nft_program: config.nft_program,
        min_block_side_length: config.min_block_side_length,
        painting: config.painting,
        pixel_price: config.pixel_price,
        resale_commission_percentage: config.resale_commission_percentage,
        resolution: config.resolution,
        ..Default::default()
    };
    unsafe {
        PROGRAM = Some(program);
    }
}

#[async_main]
async fn main() {
    let action: NFTPixelboardAction = msg::load().expect("Unable to decode `NFTPixelboardAction`");
    let program = unsafe { PROGRAM.get_or_insert(Default::default()) };
    match action {
        NFTPixelboardAction::Mint {
            rectangle,
            token_metadata,
            painting,
        } => program.mint(rectangle, token_metadata, painting).await,
        NFTPixelboardAction::Buy(token_id) => program.buy(token_id).await,
        NFTPixelboardAction::PutUpForSale {
            token_id,
            pixel_price,
        } => program.put_up_for_sale(token_id, pixel_price).await,
        NFTPixelboardAction::Paint { token_id, painting } => program.paint(token_id, painting),
    }
}

#[no_mangle]
extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: NFTPixelboardState = msg::load().expect("Unable to decode `NFTPixelboardState`");
    let program = unsafe { PROGRAM.get_or_insert(Default::default()) };
    let encoded = match state {
        NFTPixelboardState::Painting => NFTPixelboardStateReply::Painting(program.painting.clone()),
        NFTPixelboardState::Resolution => NFTPixelboardStateReply::Resolution(program.resolution),
        NFTPixelboardState::PixelPrice => NFTPixelboardStateReply::PixelPrice(program.pixel_price),
        NFTPixelboardState::MinBlockSideLength => {
            NFTPixelboardStateReply::MinBlockSideLength(program.min_block_side_length)
        }
        NFTPixelboardState::ResaleCommissionPercentage => {
            NFTPixelboardStateReply::ResaleCommissionPercentage(
                program.resale_commission_percentage,
            )
        }
        NFTPixelboardState::PixelInfo(coordinates) => {
            let mut token = Default::default();

            if coordinates.x < program.resolution.width && coordinates.y < program.resolution.height
            {
                let dot: Rectangle = (
                    (coordinates.x, coordinates.y + 1),
                    (coordinates.x, coordinates.y),
                )
                    .into();

                if let Some((rectangle, token_info)) =
                    program.tokens_by_rectangles.range(..dot).next_back()
                {
                    if coordinates.x < rectangle.lower_right_corner.x
                        && coordinates.y < rectangle.lower_right_corner.y
                    {
                        token = Token(*rectangle, *token_info)
                    }
                }
            }

            NFTPixelboardStateReply::PixelInfo(token)
        }
        NFTPixelboardState::TokenInfo(token_id) => {
            let mut token = Default::default();

            if let Some(rectangle) = program.rectangles_by_token_ids.get(&token_id) {
                if let Some(token_info) = program.tokens_by_rectangles.get(rectangle) {
                    token = Token(*rectangle, *token_info);
                }
            }

            NFTPixelboardStateReply::TokenInfo(token)
        }
        NFTPixelboardState::FTProgram => NFTPixelboardStateReply::FTProgram(program.ft_program),
        NFTPixelboardState::NFTProgram => NFTPixelboardStateReply::NFTProgram(program.nft_program),
    }
    .encode();
    gstd::util::to_leak_ptr(encoded)
}

gstd::metadata! {
    title: "NFT pixelboard",
    init:
        input: InitNFTPixelboard,
    handle:
        input: NFTPixelboardAction,
        output: NFTPixelboardEvent,
    state:
        input: NFTPixelboardState,
        output: NFTPixelboardStateReply,
}
