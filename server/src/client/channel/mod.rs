use crate::common::error::Result as CommonResult;
use crate::common::message::ResponseMessage;

mod default;

pub use default::DefaultClientChannel;

pub trait ClientChannel {
    /// Send ResponseMessage unconditionally, ignoring any potential errors.
    fn do_send(&self, msg: ResponseMessage) -> CommonResult<()>;

    /// Tries to send ResponseMessage.
    fn try_send(&self, msg: ResponseMessage) -> CommonResult<()>;

    // TODO: Implement send method which provides mechanism to track for
    // message delivery
}
