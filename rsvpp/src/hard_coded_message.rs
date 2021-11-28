use crate::{
    message::{MessageClientId, MessageContext, MessageId, MessageName},
    pack::Pack,
};

pub const VL_API_SOCK_CLNT_CREATE_MSG_ID: u16 = 15;
pub const VL_API_SOCK_CLNT_CREATE_REP_MSG_ID: u16 = 16;

#[derive(Pack, Debug, PartialEq, Eq)]
#[packed]
pub struct ApiMessageReplyHeader {
    pub _vl_msg_id: u16,
    pub context: u32,
}

#[derive(Pack, Debug, PartialEq, Eq, Default)]
#[packed]
pub struct VlApiSockclntCreateT {
    pub _vl_msg_id: u16,
    pub context: u32,
    #[len(64)]
    pub name: String,
}

impl MessageName for VlApiSockclntCreateT {
    fn message_name() -> String {
        "sockclnt_create".to_string()
    }
}

impl MessageId for VlApiSockclntCreateT {
    fn message_id(&self) -> u16 {
        self._vl_msg_id
    }

    fn set_message_id(mut self, id: u16) -> Self {
        self._vl_msg_id = id;
        self
    }
}

impl MessageContext for VlApiSockclntCreateT {
    fn context(&self) -> u32 {
        self.context
    }

    fn set_context(mut self, ctx: u32) -> Self {
        self.context = ctx;
        self
    }
}

#[derive(Pack, Debug, PartialEq, Eq)]
#[packed]
pub struct VlApiSockclntCreateReplyT {
    pub _vl_msg_id: u16,
    pub client_index: u32,
    pub context: u32,
    pub response: i32,
    pub index: u32,
    pub count: u16,
    #[len("count")]
    pub message_table: Vec<VlApiMessageTableEntryT>,
}

impl MessageName for VlApiSockclntCreateReplyT {
    fn message_name() -> String {
        "sockclnt_create_reply".to_string()
    }
}

impl MessageId for VlApiSockclntCreateReplyT {
    fn message_id(&self) -> u16 {
        self._vl_msg_id
    }

    fn set_message_id(mut self, id: u16) -> Self {
        self._vl_msg_id = id;
        self
    }
}

impl MessageContext for VlApiSockclntCreateReplyT {
    fn context(&self) -> u32 {
        self.context
    }

    fn set_context(mut self, ctx: u32) -> Self {
        self.context = ctx;
        self
    }
}

impl MessageClientId for VlApiSockclntCreateReplyT {
    fn client_index(&self) -> u32 {
        self.client_index
    }

    fn set_client_index(mut self, idx: u32) -> Self {
        self.client_index = idx;
        self
    }
}

#[derive(Pack, Debug, PartialEq, Eq)]
#[packed]
pub struct VlApiMessageTableEntryT {
    pub index: u16,
    #[len(64)]
    pub name: String,
}
