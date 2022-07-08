use ft_io::{FTAction, FTEvent};
use gstd::{msg, ActorId};
use nft_pixelboard_io::*;

pub fn reply(nft_pixelboard_event: NFTPixelboardEvent) {
    msg::reply(nft_pixelboard_event, 0).expect("Error during a replying with NFTPixelboardEvent");
}

pub async fn transfer_tokens(ft_program: ActorId, from: ActorId, to: ActorId, amount: u128) {
    msg::send_for_reply_as::<_, FTEvent>(ft_program, FTAction::Transfer { from, to, amount }, 0)
        .expect("Error during a sending FTAction::Transfer to a FT program")
        .await
        .expect("Unable to decode FTEvent");
}
