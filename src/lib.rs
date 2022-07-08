#![no_std]

use core::ops::RangeTo;

use gear_lib::non_fungible_token::{
    io::NFTTransfer,
    token::{TokenId, TokenMetadata},
};
use gstd::{async_main, msg, prelude::*, ActorId};
use nft_io::NFTAction;
use nft_pixelboard_io::*;

mod util;
use util::*;

fn get_pixel_count<P: Into<usize>>(width: P, height: P) -> usize {
    width
        .into()
        .checked_mul(height.into())
        .expect("Pixel count overflow")
}

fn check_painting(painting: &Vec<Color>, pixel_count: usize) {
    if painting.len() != pixel_count {
        panic!("Pixel count in a painting must equal the count in a canvas/NFT")
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

    tokens_by_rectangles: BTreeMap<Rectangle, TokenInfo>,
    tokens_by_ids: BTreeMap<TokenId, Rectangle>,

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

        let rectangle_width =
            (rectangle.lower_right_corner.x - rectangle.upper_left_corner.x) as usize;
        let rectangle_height =
            (rectangle.lower_right_corner.y - rectangle.upper_left_corner.y) as usize;

        let rectangle_pixel_count = get_pixel_count(rectangle_width, rectangle_height);

        transfer_tokens(
            self.ft_program,
            msg::source(),
            self.owner,
            (rectangle_pixel_count as u128)
                .checked_mul(self.pixel_price)
                .expect("Pixel price can't be more than 2^96"),
        )
        .await;

        let raw_reply: Vec<u8> =
            msg::send_for_reply_as(self.nft_program, NFTAction::Mint { token_metadata }, 0)
                .expect("Error during a sending NFTAction::Mint to an NFT program")
                .await
                .expect("Unable to decode Vec<u8>");

        let decoded_reply =
            NFTTransfer::decode(&mut &raw_reply[..]).expect("Unable to decode NFTTransfer");

        // Painting

        check_painting(&painting, rectangle_pixel_count);

        let canvas_width = self.resolution.width as usize;

        let first_row_end = canvas_width * rectangle.upper_left_corner.y as usize
            + rectangle.lower_right_corner.x as usize;
        let first_row_start = first_row_end - rectangle_width;

        let (first_row_painting, rest_of_painting) = painting.split_at(rectangle_width);
        self.painting[first_row_start..first_row_end].copy_from_slice(first_row_painting);

        for (canvas_row, painting_row) in self.painting
            [first_row_end..first_row_end + (rectangle_height - 1) * canvas_width]
            .chunks_exact_mut(self.resolution.width as _)
            .zip(rest_of_painting.chunks_exact(rectangle_width))
        {
            canvas_row[canvas_width - rectangle_width..].copy_from_slice(painting_row);
        }

        // Insertion and replying

        self.tokens_by_rectangles.insert(
            rectangle,
            TokenInfo {
                owner: msg::source(),
                pixel_price: None,
                token_id: decoded_reply.token_id,
            },
        );
        self.tokens_by_ids.insert(decoded_reply.token_id, rectangle);

        reply(NFTPixelboardEvent::Minted(decoded_reply.token_id));
    }

    async fn buy(&mut self, token_id: TokenId) {
        let rectangle = self
            .tokens_by_ids
            .get(&token_id)
            .expect("NFT not found by ID");
        let token = self
            .tokens_by_rectangles
            .get_mut(rectangle)
            .expect("NFT not found by the rectangle");

        let pixel_price = if let Some(price) = token.pixel_price {
            price
        } else {
            panic!("NFT isn't for sale");
        };

        transfer_tokens(
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

        msg::send_for_reply_as::<_, NFTTransfer>(
            self.nft_program,
            NFTAction::Transfer {
                to: msg::source(),
                token_id,
            },
            0,
        )
        .expect("Error during a sending NFTAction::Transfer to an NFT program")
        .await
        .expect("Unable to decode NFTTransfer");

        token.pixel_price = None;
        token.owner = msg::source();

        reply(NFTPixelboardEvent::Bought(token_id));
    }

    fn put_up_for_sale(&mut self, token_id: TokenId, pixel_price: u128) {
        if pixel_price > 2u128.pow(96) {
            panic!("Pixel price can't be more than 2^96");
        }

        let rectangle = self
            .tokens_by_ids
            .get(&token_id)
            .expect("NFT not found by ID");
        let token = self
            .tokens_by_rectangles
            .get_mut(rectangle)
            .expect("NFT not found by the rectangle");

        assert_eq!(token.owner, msg::source());

        token.pixel_price = Some(pixel_price);

        reply(NFTPixelboardEvent::ForSale(token_id));
    }

    fn paint(&mut self, token_id: TokenId, painting: Vec<Color>) {
        let rectangle = self
            .tokens_by_ids
            .get(&token_id)
            .expect("NFT not found by ID");
        let token = self
            .tokens_by_rectangles
            .get_mut(rectangle)
            .expect("NFT not found by the rectangle");

        assert_eq!(token.owner, msg::source());

        check_painting(
            &painting,
            get_pixel_count(
                rectangle.lower_right_corner.x - rectangle.upper_left_corner.x,
                rectangle.lower_right_corner.y - rectangle.upper_left_corner.y,
            ),
        );

        let rectangle_width =
            (rectangle.lower_right_corner.x - rectangle.upper_left_corner.x) as usize;
        let rectangle_height =
            (rectangle.lower_right_corner.y - rectangle.upper_left_corner.y) as usize;

        let canvas_width = self.resolution.width as usize;

        let first_row_end = canvas_width * rectangle.upper_left_corner.y as usize
            + rectangle.lower_right_corner.x as usize;
        let first_row_start = first_row_end - rectangle_width;

        let (first_row_painting, rest_of_painting) = painting.split_at(rectangle_width);
        self.painting[first_row_start..first_row_end].copy_from_slice(first_row_painting);

        for (canvas_row, painting_row) in self.painting
            [first_row_end..first_row_end + (rectangle_height - 1) * canvas_width]
            .chunks_exact_mut(self.resolution.width as _)
            .zip(rest_of_painting.chunks_exact(rectangle_width))
        {
            canvas_row[canvas_width - rectangle_width..].copy_from_slice(painting_row);
        }
    }
}

static mut PROGRAM: Option<NFTPixelboard> = None;

#[no_mangle]
extern "C" fn init() {
    let config: InitNFTPixelboard = msg::load().expect("Unable to decode InitNFTPixelboard");

    if config.ft_program == ActorId::zero() {
        panic!("FT program address can't be 0");
    }

    if config.nft_program == ActorId::zero() {
        panic!("NFT program address can't be 0");
    }

    if config.min_block_side_length == 0 {
        panic!("min_block_side_length must be greater than 0");
    }

    if config.resolution.width == 0 || config.resolution.height == 0 {
        panic!("Each side of resolution must be greater than 0");
    }

    if config.resolution.width % config.min_block_side_length != 0
        || config.resolution.height % config.min_block_side_length != 0
    {
        panic!("Each side of resolution must be a multiple of min_block_side_length");
    }

    if config.resale_commission_percentage > 100 {
        panic!("resale_commission_percentage must be equal to or less than 100");
    }

    if config.pixel_price > 2u128.pow(96) {
        panic!("Pixel price can't be more than 2^96");
    }

    check_painting(
        &config.painting,
        get_pixel_count(config.resolution.width, config.resolution.height),
    );

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
    let action: NFTPixelboardAction = msg::load().expect("Unable to decode NFTPixelboardAction");
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
        } => program.put_up_for_sale(token_id, pixel_price),
        NFTPixelboardAction::Paint { token_id, painting } => program.paint(token_id, painting),
    }
}

#[no_mangle]
extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: NFTPixelboardState = msg::load().expect("Unable to decode NFTPixelboardState");
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
            let token = program
                .tokens_by_rectangles
                .range::<Rectangle, RangeTo<Rectangle>>(
                    ..(
                        (coordinates.x, coordinates.y),
                        (coordinates.x, coordinates.y),
                    )
                        .into(),
                )
                .next_back()
                .unwrap();
            NFTPixelboardStateReply::PixelInfo(Token(*token.0, *token.1))
        }
        NFTPixelboardState::TokenInfo(token_id) => {
            let rectangle = program
                .tokens_by_ids
                .get(&token_id)
                .expect("NFT not found by ID");
            let token = program
                .tokens_by_rectangles
                .get(rectangle)
                .expect("NFT not found by the rectangle");

            NFTPixelboardStateReply::TokenInfo(Token(*rectangle, *token))
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
