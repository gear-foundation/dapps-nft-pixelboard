#![no_std]

use gear_lib::non_fungible_token::token::{TokenId, TokenMetadata};
use gstd::{prelude::*, ActorId};

pub type MinBlockSideLength = u16;
pub type Color = u8;

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub struct Rectangle {
    pub upper_left_corner: Coordinates,
    pub lower_right_corner: Coordinates,
}

impl Rectangle {
    pub fn width(&self) -> MinBlockSideLength {
        self.lower_right_corner.x - self.upper_left_corner.x
    }

    pub fn height(&self) -> MinBlockSideLength {
        self.lower_right_corner.y - self.upper_left_corner.y
    }
}

impl
    From<(
        (MinBlockSideLength, MinBlockSideLength),
        (MinBlockSideLength, MinBlockSideLength),
    )> for Rectangle
{
    fn from(
        rectangle: (
            (MinBlockSideLength, MinBlockSideLength),
            (MinBlockSideLength, MinBlockSideLength),
        ),
    ) -> Self {
        Self {
            upper_left_corner: rectangle.0.into(),
            lower_right_corner: rectangle.1.into(),
        }
    }
}

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub struct Coordinates {
    pub x: MinBlockSideLength,
    pub y: MinBlockSideLength,
}

impl From<(MinBlockSideLength, MinBlockSideLength)> for Coordinates {
    fn from(coordinates: (MinBlockSideLength, MinBlockSideLength)) -> Self {
        Self {
            x: coordinates.0,
            y: coordinates.1,
        }
    }
}

#[derive(Decode, Encode, Default, Clone, Copy, TypeInfo)]
pub struct Resolution {
    pub width: MinBlockSideLength,
    pub height: MinBlockSideLength,
}

impl From<(MinBlockSideLength, MinBlockSideLength)> for Resolution {
    fn from(resolution: (MinBlockSideLength, MinBlockSideLength)) -> Self {
        Self {
            width: resolution.0,
            height: resolution.1,
        }
    }
}

#[derive(Decode, Encode, TypeInfo, Clone, Copy, Debug, Default)]
pub struct Token(pub Rectangle, pub TokenInfo);

#[derive(Decode, Encode, TypeInfo, Clone, Copy, Debug, Default)]
pub struct TokenInfo {
    pub token_id: TokenId,
    pub owner: ActorId,
    pub pixel_price: Option<u128>,
}

#[derive(Decode, Encode, TypeInfo)]
pub struct InitNFTPixelboard {
    pub owner: ActorId,
    pub min_block_side_length: MinBlockSideLength,
    pub pixel_price: u128,
    pub resolution: Resolution,
    pub resale_commission_percentage: u8,
    pub painting: Vec<Color>,

    pub ft_program: ActorId,
    pub nft_program: ActorId,
}

#[derive(Decode, Encode, TypeInfo)]
pub enum NFTPixelboardAction {
    Mint {
        rectangle: Rectangle,
        token_metadata: TokenMetadata,
        painting: Vec<Color>,
    },
    Buy(TokenId),
    PutUpForSale {
        token_id: TokenId,
        pixel_price: u128,
    },
    Paint {
        token_id: TokenId,
        painting: Vec<Color>,
    },
}

#[derive(Decode, Encode, TypeInfo)]
pub enum NFTPixelboardEvent {
    Minted(TokenId),
    Bought(TokenId),
    ForSale(TokenId),
    Painted(TokenId),
}

#[derive(Decode, Encode, TypeInfo)]
pub enum NFTPixelboardState {
    Painting,
    Resolution,
    PixelPrice,
    MinBlockSideLength,
    PixelInfo(Coordinates),
    TokenInfo(TokenId),
    ResaleCommissionPercentage,
    FTProgram,
    NFTProgram,
}

#[derive(Decode, Encode, TypeInfo)]
pub enum NFTPixelboardStateReply {
    Painting(Vec<Color>),
    Resolution(Resolution),
    PixelPrice(u128),
    MinBlockSideLength(MinBlockSideLength),
    PixelInfo(Token),
    TokenInfo(Token),
    ResaleCommissionPercentage(u8),
    FTProgram(ActorId),
    NFTProgram(ActorId),
}
