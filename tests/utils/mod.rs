pub const FOREIGN_USER: u64 = 12345;
pub const OWNER: u64 = 54321;
pub use gstd::prelude::*;
pub use nft_pixelboard_io::*;

use core::fmt::Debug;
use ft_io::{FTAction, FTEvent, InitConfig as InitFT};
use gear_lib::non_fungible_token::state::{NFTQuery, NFTQueryReply};
use gstd::ActorId;
use gtest::{Log, Program as InnerProgram, RunResult, System};
use nft_io::InitNFT;

pub fn initialize_system() -> System {
    let system = System::new();
    system.init_logger();
    system
}

pub trait Program {
    fn inner_program(&self) -> &InnerProgram;

    fn actor_id(&self) -> ActorId {
        self.inner_program().id().as_ref().try_into().unwrap()
    }
}

pub struct FungibleToken<'a>(InnerProgram<'a>);

impl Program for FungibleToken<'_> {
    fn inner_program(&self) -> &InnerProgram {
        &self.0
    }
}

impl<'a> FungibleToken<'a> {
    pub fn initialize(system: &'a System) -> Self {
        let program = InnerProgram::from_file(system, "./target/fungible_token.wasm");

        assert!(!program
            .send(
                FOREIGN_USER,
                InitFT {
                    name: Default::default(),
                    symbol: Default::default(),
                },
            )
            .main_failed());

        Self(program)
    }

    pub fn mint(&self, from: u64, amount: u128) {
        assert!(self
            .0
            .send(from, FTAction::Mint(amount))
            .contains(&Log::builder().payload(FTEvent::Transfer {
                amount,
                from: ActorId::zero(),
                to: from.into(),
            })))
    }

    pub fn balance(&self, actor_id: u64) -> FTActionBalance {
        FTActionBalance(
            self.0
                .send(FOREIGN_USER, FTAction::BalanceOf(actor_id.into())),
        )
    }
}

pub struct FTActionBalance(RunResult);

impl FTActionBalance {
    pub fn check(self, balance: u128) {
        assert!(self
            .0
            .contains(&Log::builder().payload(FTEvent::Balance(balance))))
    }
}

pub struct NonFungibleToken<'a>(InnerProgram<'a>);

impl Program for NonFungibleToken<'_> {
    fn inner_program(&self) -> &InnerProgram {
        &self.0
    }
}

impl<'a> NonFungibleToken<'a> {
    pub fn initialize(system: &'a System) -> Self {
        let program = InnerProgram::from_file(system, "./target/nft.wasm");

        assert!(!program
            .send(
                FOREIGN_USER,
                InitNFT {
                    base_uri: Default::default(),
                    name: Default::default(),
                    royalties: Default::default(),
                    symbol: Default::default(),
                }
            )
            .main_failed());

        Self(program)
    }

    pub fn meta_state(&self) -> NonFungibleTokenMetaState {
        NonFungibleTokenMetaState(&self.0)
    }
}

pub struct NonFungibleTokenMetaState<'a>(&'a InnerProgram<'a>);

impl NonFungibleTokenMetaState<'_> {
    pub fn owner(self, token_id: u128) -> NonFungibleTokenMetaStateReply<ActorId> {
        if let NFTQueryReply::Token { token: reply } = self
            .0
            .meta_state(NFTQuery::Token {
                token_id: token_id.into(),
            })
            .unwrap()
        {
            NonFungibleTokenMetaStateReply(reply.owner_id)
        } else {
            unreachable!();
        }
    }
}

pub struct NonFungibleTokenMetaStateReply<T>(T);

impl<T: Debug + PartialEq> NonFungibleTokenMetaStateReply<T> {
    pub fn check(self, value: T) {
        assert_eq!(self.0, value);
    }
}

pub struct NFTPixelboard<'a>(InnerProgram<'a>);

impl<'a> NFTPixelboard<'a> {
    pub fn initialize(
        system: &'a System,
        ft_program: ActorId,
        nft_program: ActorId,
    ) -> NFTPixelboardInit {
        let program = InnerProgram::current(system);

        let failed = program
            .send(
                FOREIGN_USER,
                InitNFTPixelboard {
                    ft_program,
                    min_block_side_length: 10,
                    nft_program,
                    owner: OWNER.into(),
                    painting: vec![0; 100],
                    pixel_price: MAX_PIXEL_PRICE,
                    resale_commission_percentage: 100,
                    resolution: (10, 10).into(),
                },
            )
            .main_failed();

        NFTPixelboardInit(program, failed)
    }

    pub fn initialize_custom(system: &'a System, config: InitNFTPixelboard) -> NFTPixelboardInit {
        let program = InnerProgram::current(system);

        let failed = program.send(FOREIGN_USER, config).main_failed();

        NFTPixelboardInit(program, failed)
    }

    pub fn meta_state(&self) -> NFTPixelboardMetaState {
        NFTPixelboardMetaState(&self.0)
    }

    pub fn mint(
        &self,
        from: u64,
        painting: Vec<Color>,
        rectangle: Rectangle,
    ) -> NFTPixelboardActionMint {
        NFTPixelboardActionMint(self.0.send(
            from,
            NFTPixelboardAction::Mint {
                painting,
                rectangle,
                token_metadata: Default::default(),
            },
        ))
    }
}

pub struct NFTPixelboardInit<'a>(InnerProgram<'a>, bool);

impl<'a> NFTPixelboardInit<'a> {
    pub fn failed(self) {
        assert!(self.1)
    }

    pub fn succeed(self) -> NFTPixelboard<'a> {
        assert!(!self.1);
        NFTPixelboard(self.0)
    }
}

pub struct NFTPixelboardMetaState<'a>(&'a InnerProgram<'a>);

impl NFTPixelboardMetaState<'_> {
    pub fn ft_program(self) -> NFTPixelboardMetaStateReply<ActorId> {
        if let NFTPixelboardStateReply::FTProgram(reply) =
            self.0.meta_state(NFTPixelboardState::FTProgram).unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn nft_program(self) -> NFTPixelboardMetaStateReply<ActorId> {
        if let NFTPixelboardStateReply::NFTProgram(reply) =
            self.0.meta_state(NFTPixelboardState::NFTProgram).unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn min_block_side_length(self) -> NFTPixelboardMetaStateReply<MinBlockSideLength> {
        if let NFTPixelboardStateReply::MinBlockSideLength(reply) = self
            .0
            .meta_state(NFTPixelboardState::MinBlockSideLength)
            .unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn painting(self) -> NFTPixelboardMetaStateReply<Vec<Color>> {
        if let NFTPixelboardStateReply::Painting(reply) =
            self.0.meta_state(NFTPixelboardState::Painting).unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn pixel_info(self, coordinates: Coordinates) -> NFTPixelboardMetaStateReply<Token> {
        if let NFTPixelboardStateReply::PixelInfo(reply) = self
            .0
            .meta_state(NFTPixelboardState::PixelInfo(coordinates))
            .unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn pixel_price(self) -> NFTPixelboardMetaStateReply<u128> {
        if let NFTPixelboardStateReply::PixelPrice(reply) =
            self.0.meta_state(NFTPixelboardState::PixelPrice).unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn resale_commission_percentage(self) -> NFTPixelboardMetaStateReply<u8> {
        if let NFTPixelboardStateReply::ResaleCommissionPercentage(reply) = self
            .0
            .meta_state(NFTPixelboardState::ResaleCommissionPercentage)
            .unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn resolution(self) -> NFTPixelboardMetaStateReply<Resolution> {
        if let NFTPixelboardStateReply::Resolution(reply) =
            self.0.meta_state(NFTPixelboardState::Resolution).unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }

    pub fn token_info(self, token_id: u128) -> NFTPixelboardMetaStateReply<Token> {
        if let NFTPixelboardStateReply::TokenInfo(reply) = self
            .0
            .meta_state(NFTPixelboardState::TokenInfo(token_id.into()))
            .unwrap()
        {
            NFTPixelboardMetaStateReply(reply)
        } else {
            unreachable!();
        }
    }
}

pub struct NFTPixelboardMetaStateReply<T>(T);

impl<T: Debug + PartialEq> NFTPixelboardMetaStateReply<T> {
    pub fn check(self, value: T) {
        assert_eq!(self.0, value);
    }
}

pub struct NFTPixelboardActionMint(RunResult);

impl NFTPixelboardActionMint {
    pub fn failed(self) {
        assert!(self.0.main_failed())
    }

    pub fn check(self, token_id: u128) {
        assert!(self
            .0
            .contains(&Log::builder().payload(NFTPixelboardEvent::Minted(token_id.into()))))
    }
}
