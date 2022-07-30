#![no_std]

use gear_lib::non_fungible_token::token::{TokenId, TokenMetadata};
use gstd::{prelude::*, ActorId};

/// The maximum price that can be set to a pixel.
///
/// This number is calculated to avoid an overflow and precisely calculate a
/// resale commission. Here's an explanation.
///
/// The maximum number of pixels that a canvas can contain is
/// [`BlockSideLength::MAX`]² = 2³². So the maximum price that each pixel
/// can have is [`u128::MAX`] / [`BlockSideLength::MAX`]² = 2⁹⁶.
///
/// To calculate the commission, the number can be multiplied by 100, so, to
/// avoid an overflow, the number must be divided by 100. Hence 2⁹⁶ / 100.
pub const MAX_PIXEL_PRICE: u128 = 2u128.pow(96) / 100;

/// A block side length.
///
/// It's also used to store pixel [`Coordinates`], [`Resolution`] of a canvas,
/// and token [`Rectangle`]s.
pub type BlockSideLength = u16;
/// A pixel color.
pub type Color = u8;

/// Coordinates of the corners of a token rectangle on a canvas.
#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub struct Rectangle {
    pub top_left_corner: Coordinates,
    pub bottom_right_corner: Coordinates,
}

impl Rectangle {
    pub fn width(&self) -> BlockSideLength {
        self.bottom_right_corner.x - self.top_left_corner.x
    }

    pub fn height(&self) -> BlockSideLength {
        self.bottom_right_corner.y - self.top_left_corner.y
    }
}

impl
    From<(
        (BlockSideLength, BlockSideLength),
        (BlockSideLength, BlockSideLength),
    )> for Rectangle
{
    fn from(
        rectangle: (
            (BlockSideLength, BlockSideLength),
            (BlockSideLength, BlockSideLength),
        ),
    ) -> Self {
        Self {
            top_left_corner: rectangle.0.into(),
            bottom_right_corner: rectangle.1.into(),
        }
    }
}

/// Coordinates of some pixel on a canvas.
#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub struct Coordinates {
    pub x: BlockSideLength,
    pub y: BlockSideLength,
}

impl From<(BlockSideLength, BlockSideLength)> for Coordinates {
    fn from((x, y): (BlockSideLength, BlockSideLength)) -> Self {
        Self { x, y }
    }
}

/// A resolution of a canvas.
#[derive(Decode, Encode, Default, Clone, Copy, TypeInfo, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub width: BlockSideLength,
    pub height: BlockSideLength,
}

impl From<(BlockSideLength, BlockSideLength)> for Resolution {
    fn from((width, height): (BlockSideLength, BlockSideLength)) -> Self {
        Self { width, height }
    }
}

/// An NFT with its [`Rectangle`] and [`TokenInfo`].
#[derive(Decode, Encode, TypeInfo, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Token(pub Rectangle, pub TokenInfo);

/// NFT info.
#[derive(Decode, Encode, TypeInfo, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenInfo {
    pub token_id: TokenId,
    pub owner: ActorId,
    /// If this field is [`None`], then this token isn't for sale, and vice
    /// versa.
    ///
    /// To calculate a price of the entire token, its area must be calculated
    /// and multiplied by `pixel_price`. The area can be calculated by
    /// multiplying a [width](`Rectangle::width`) &
    /// [height](`Rectangle::height`) from token [`Rectangle`]. Token
    /// [`Rectangle`] can be obtained by [`NFTPixelboardState::TokenInfo`]
    /// using `token_id` from this struct.
    pub pixel_price: Option<u128>,
}

/// Initializes the NFT pixelboard program.
///
/// # Requirements
/// * `owner` address mustn't be 0.
/// * `block_side_length` must be more than 0.
/// * `pixel_price` mustn't be more than [`MAX_PIXEL_PRICE`].
/// * A [width](`Resolution#structfield.width`) &
/// [height](`Resolution#structfield.height`) (`resolution`) of a canvas must
/// be more than 0.
/// * Each side of `resolution` must be a multiple of `block_side_length`.
/// * `painting` length must equal a pixel count in a canvas (which can be
/// calculated by multiplying a [width](`Resolution#structfield.width`) &
/// [height](`Resolution#structfield.height`) from `resolution`).
/// * `commission_percentage` mustn't be more than 100.
/// * `ft_program` address mustn't be 0.
/// * `nft_program` address mustn't be 0.
#[derive(Decode, Encode, TypeInfo, Clone)]
pub struct InitNFTPixelboard {
    /// An address of a pixelboard owner to which minting fees and commissions
    /// on resales will be transferred.
    pub owner: ActorId,
    /// A block side length.
    ///
    /// To avoid a canvas clogging with one pixel NFTs, blocks are used instead
    /// of pixels to set token [`Rectangle`]s. This parameter is used to set a
    /// side length of these pixel blocks. If blocks aren't needed,
    /// then this parameter can be set to 1, so the block side length
    /// will equal a pixel.
    pub block_side_length: BlockSideLength,
    /// The price of a free pixel. It'll be used to calculate a minting price.
    pub pixel_price: u128,
    /// A canvas (pixelboard) [width](`Resolution#structfield.width`) &
    /// [height](`Resolution#structfield.height`).
    pub resolution: Resolution,
    /// A commission percentage that'll be included in each token resale.
    pub commission_percentage: u8,
    /// A painting that'll be displayed on the free territory of a pixelboard.
    pub painting: Vec<Color>,

    /// A FT program address.
    pub ft_program: ActorId,
    /// An NFT program address.
    pub nft_program: ActorId,
}

/// Sends a program info about what it should do.
#[derive(Decode, Encode, TypeInfo, Clone)]
pub enum NFTPixelboardAction {
    /// Mints one NFT on a pixelboard with given `token_metadata`.
    ///
    /// Transfers a minted NFT to [`msg::source()`].
    ///
    /// # Requirements
    /// * `rectangle` coordinates mustn't be out of a canvas.
    /// * `rectangle` coordinates mustn't be mixed up or belong to wrong
    /// corners.
    /// * `rectangle` coordinates must observe a block layout. In
    /// other words, each `rectangle` coordinate must be a multiple of a
    /// block side length in the canvas. The block side length can be
    /// obtained by [`NFTPixelboardState::BlockSideLength`].
    /// * NFT `rectangle` mustn't collide with already minted one.
    /// * `painting` length must equal a pixel count in an NFT
    /// (which can be calculated by multiplying a [width](`Rectangle::width`)
    /// & [height](`Rectangle::height`) from `rectangle`).
    /// * [`msg::source()`] must have enough tokens to buy all free pixels that
    /// `rectangle` will occupy. An enough number of tokens can be calculated by
    /// multiplying a `rectangle` area and the price of a free pixel. The area
    /// can be calculated by multiplying a [width](`Rectangle::width`) &
    /// [height](`Rectangle::height`) from `rectangle`. The price of a free
    /// pixel can be obtained by [`NFTPixelboardState::PixelPrice`].
    ///
    /// On success, returns [`NFTPixelboardEvent::Minted`].
    ///
    /// [`msg::source()`]: gstd::msg::source
    Mint {
        rectangle: Rectangle,
        token_metadata: TokenMetadata,
        /// A painting that'll be displayed in a place of an NFT on a pixelboard
        /// after a successful minting.
        painting: Vec<Color>,
    },

    /// Buys an NFT minted on a pixelboard.
    ///
    /// Transfers a purchased NFT from a pixelboard to [`msg::source()`].
    ///
    /// **Note:** If [`msg::source()`] has enough tokens to pay a resale
    /// commission but not the entire token, then the commission will still be
    /// withdrawn from its account.
    ///
    /// # Requirements
    /// * An NFT must be minted on a pixelboard.
    /// * An NFT must be for sale. This can be found out by
    /// [`NFTPixelboardState::TokenInfo`]. See also the documentation of
    /// [`TokenInfo#structfield.pixel_price`].
    /// * [`msg::source()`] must have enough tokens to buy all pixels that a
    /// token occupies. This can be found out by
    /// [`NFTPixelboardState::TokenInfo`]. See also the documentation of
    /// [`TokenInfo#structfield.pixel_price`].
    ///
    /// On success, returns [`NFTPixelboardEvent::Bought`].
    ///
    /// [`msg::source()`]: gstd::msg::source
    Buy(TokenId),

    /// Changes a sale state of an NFT minted on a pixelboard.
    ///
    /// There are 3 options of a sale state change:
    /// * Putting up for sale\
    /// If an NFT is **not** for sale, then assigning `pixel_price` to [`Some`]
    /// price will transfer it to a pixelboard program & put it up for sale.
    /// * Updating a pixel price\
    /// If an NFT is for sale, then assigning `pixel_price` to [`Some`] price
    /// will update its pixel price.
    /// * Removing from sale\
    /// Assigning the `pixel_price` to [`None`] will transfer an NFT back to its
    /// owner & remove an NFT from sale.
    ///
    /// **Note:** A commission is included in each token resale, so a seller
    /// will receive not all tokens but tokens with a commission deduction. A
    /// commission percentage can be obtained by
    /// [`NFTPixelboardState::CommissionPercentage`].
    ///
    /// # Requirements
    /// * An NFT must be minted on a pixelboard.
    /// * [`msg::source()`](gstd::msg::source) must be the owner of an NFT.
    /// * `pixel_price` mustn't be more than [`MAX_PIXEL_PRICE`].
    ///
    /// On success, returns [`NFTPixelboardEvent::SaleStateChanged`].
    ChangeSaleState {
        token_id: TokenId,
        /// A price of each pixel that an NFT occupies. To calculate a price of
        /// the entire NFT, see the documentation of
        /// [`TokenInfo#structfield.pixel_price`].
        pixel_price: Option<u128>,
    },

    /// Paints an NFT minted on a pixelboard.
    ///
    /// # Requirements
    /// * An NFT must be minted on a pixelboard.
    /// * [`msg::source()`](gstd::msg::source) must be the owner of an NFT.
    /// * `painting` length must equal a pixel count in an NFT. The count can be
    /// calculated by multiplying a [width](`Rectangle::width`) &
    /// [height](`Rectangle::height`) from a rectangle of the NFT. The NFT
    /// rectangle can be obtained by [`NFTPixelboardState::TokenInfo`].
    ///
    /// On success, returns [`NFTPixelboardEvent::Painted`].
    Paint {
        token_id: TokenId,
        painting: Vec<Color>,
    },
}

/// A result of processed [`NFTPixelboardAction`].
#[derive(Decode, Encode, TypeInfo)]
pub enum NFTPixelboardEvent {
    /// Should be returned from [`NFTPixelboardAction::Mint`].
    Minted(TokenId),
    /// Should be returned from [`NFTPixelboardAction::Buy`].
    Bought(TokenId),
    /// Should be returned from [`NFTPixelboardAction::ChangeSaleState`].
    SaleStateChanged(TokenId),
    /// Should be returned from [`NFTPixelboardAction::Paint`].
    Painted(TokenId),
}

/// Requests a program state.
#[derive(Decode, Encode, TypeInfo)]
pub enum NFTPixelboardState {
    /// Gets a painting from an entire canvas of a pixelboard.
    ///
    /// Returns [`NFTPixelboardStateReply::Painting`].
    Painting,

    /// Gets a pixelboard (canvas) resolution.
    ///
    /// Returns [`NFTPixelboardStateReply::Resolution`].
    Resolution,

    /// Gets the price of a free pixel.
    ///
    /// Returns [`NFTPixelboardStateReply::PixelPrice`].
    PixelPrice,

    /// Gets a block side length.
    ///
    /// For more info about this parameter, see
    /// [`InitNFTPixelboard#structfield.block_side_length`] documentation.
    ///
    /// Returns [`NFTPixelboardStateReply::BlockSideLength`].
    BlockSideLength,

    /// Gets [`Token`] info by pixel coordinates.
    ///
    /// Useful, for example, for inspecting a pixelboard by clicking on
    /// paintings.
    ///
    /// Returns [`NFTPixelboardStateReply::PixelInfo`].
    PixelInfo(Coordinates),

    /// Gets [`Token`] info by its ID.
    ///
    /// Returns [`NFTPixelboardStateReply::TokenInfo`].
    TokenInfo(TokenId),

    /// Gets a resale commission percentage.
    ///
    /// Returns [`NFTPixelboardStateReply::CommissionPercentage`].
    CommissionPercentage,

    /// Gets an FT program address used by a pixelboard.
    ///
    /// Returns [`NFTPixelboardStateReply::FTProgram`].
    FTProgram,

    /// Gets an NFT program address used by a pixelboard.
    ///
    /// Returns [`NFTPixelboardStateReply::NFTProgram`].
    NFTProgram,
}

/// A reply for requested [`NFTPixelboardState`].
#[derive(Decode, Encode, TypeInfo)]
pub enum NFTPixelboardStateReply {
    /// Should be returned from [`NFTPixelboardState::Painting`].
    Painting(Vec<Color>),
    /// Should be returned from [`NFTPixelboardState::Resolution`].
    Resolution(Resolution),
    /// Should be returned from [`NFTPixelboardState::PixelPrice`].
    PixelPrice(u128),
    /// Should be returned from [`NFTPixelboardState::BlockSideLength`].
    BlockSideLength(BlockSideLength),
    /// Should be returned from [`NFTPixelboardState::PixelInfo`].
    PixelInfo(Token),
    /// Should be returned from [`NFTPixelboardState::TokenInfo`].
    TokenInfo(Token),
    /// Should be returned from [`NFTPixelboardState::CommissionPercentage`].
    CommissionPercentage(u8),
    /// Should be returned from [`NFTPixelboardState::FTProgram`].
    FTProgram(ActorId),
    /// Should be returned from [`NFTPixelboardState::NFTProgram`].
    NFTProgram(ActorId),
}
