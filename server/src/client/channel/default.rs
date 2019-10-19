use actix::prelude::*;

use crate::common::error::{ErrorKind, Result as CommonResult, ContextExt};
use crate::common::message::ResponseMessage;

use super::ClientChannel;

pub struct DefaultClientChannel {
    addr: Recipient<ResponseMessage>,
}

impl ClientChannel for DefaultClientChannel {
    fn do_send(&self, msg: ResponseMessage) -> CommonResult<()> {
        self.addr.do_send(msg).context(|| {
            (
                ErrorKind::SendMessageError,
                "Error when sending response message to client channel",
            )
        })
    }

    fn try_send(&self, msg: ResponseMessage) -> CommonResult<()> {
        self.addr.try_send(msg).context(|| {
            (
                ErrorKind::SendMessageError,
                "Error when trying to send response message to client channel",
            )
        })
    }
}

impl DefaultClientChannel {
    pub fn new(addr: Recipient<ResponseMessage>) -> Self {
        DefaultClientChannel { addr }
    }
}
