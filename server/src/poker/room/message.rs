use actix::prelude::*;

use crate::client::ClientId;
use crate::common::error::Result as CommonResult;
use crate::common::message::RequestMessage;

#[derive(Message)]
#[rtype(result = "CommonResult<()>")]
pub struct ClientRequestMessage {
    pub client_id: ClientId,
    pub req: RequestMessage,
}
