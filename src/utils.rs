use ft_logic_io::Action;
use ft_main_io::{FTokenAction, FTokenEvent};
use gear_lib::non_fungible_token::token::{TokenId, TokenMetadata};
use gstd::{msg, prelude::*, ActorId};
use nft_io::{NFTAction, NFTEvent};
use nft_pixelboard_io::*;

pub async fn transfer_ftokens(
    transaction_id: u64,
    ft_contract_id: &ActorId,
    sender: &ActorId,
    recipient: &ActorId,
    amount: u128,
) -> Result<(), NFTPixelboardError> {
    let reply = msg::send_for_reply_as::<_, FTokenEvent>(
        *ft_contract_id,
        FTokenAction::Message {
            transaction_id,
            payload: Action::Transfer {
                sender: *sender,
                recipient: *recipient,
                amount,
            }
            .encode(),
        },
        0,
    )
    .expect("Error in sending a message `FTokenAction::Message`")
    .await;
    match reply {
        Ok(FTokenEvent::Ok) => Ok(()),
        _ => Err(NFTPixelboardError::FTokensTransferFailed),
    }
}

pub async fn transfer_nft(
    transaction_id: TransactionId,
    nft_program: &ActorId,
    to: &ActorId,
    token_id: TokenId,
) -> Result<(), NFTPixelboardError> {
    let reply = msg::send_for_reply_as::<_, NFTEvent>(
        *nft_program,
        NFTAction::Transfer {
            transaction_id,
            to: *to,
            token_id,
        },
        0,
    )
    .expect("Error during sending `NFTAction::Transfer` to an NFT program")
    .await;
    match reply {
        Ok(NFTEvent::Transfer(_)) => Ok(()),
        _ => Err(NFTPixelboardError::NFTTransferFailed),
    }
}

pub async fn mint_nft(
    transaction_id: TransactionId,
    nft_program: &ActorId,
    token_metadata: TokenMetadata,
) -> Result<TokenId, NFTPixelboardError> {
    let reply = msg::send_for_reply_as::<_, NFTEvent>(
        *nft_program,
        NFTAction::Mint {
            transaction_id,
            token_metadata,
        },
        0,
    )
    .expect("Error during sending `NFTAction::Mint` to an NFT program")
    .await;
    match reply {
        Ok(NFTEvent::Transfer(transfer)) => Ok(transfer.token_id),
        _ => Err(NFTPixelboardError::NFTMintFailed),
    }
}
