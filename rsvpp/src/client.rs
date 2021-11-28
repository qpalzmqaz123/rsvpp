use std::{collections::HashMap, sync::Arc};

use pack::Pack;
use tokio::sync::Mutex;

use crate::{
    hard_coded_message::{
        VlApiSockclntCreateReplyT, VlApiSockclntCreateT, VL_API_SOCK_CLNT_CREATE_MSG_ID,
        VL_API_SOCK_CLNT_CREATE_REP_MSG_ID,
    },
    message::{Message, MessageClientId, MessageContext, MessageId, MessageName},
    transport, Error, Result, Session, CLIENT_NAME,
};

#[derive(Debug)]
pub struct MessageEntry {
    id: u16,
    name: String,
    crc: String,
}

pub struct Client {
    sess: Session,
    ctx: Mutex<u32>,
    client_index: u32,
    msg_id_map: HashMap<u16, Arc<MessageEntry>>,
    msg_name_map: HashMap<String, Arc<MessageEntry>>,
}

impl Client {
    pub async fn connect_unix(sock: &str) -> Result<Self> {
        // Create transport
        log::trace!("Connect unix: '{}'", sock);
        let trans = transport::unix(sock).await?;

        // Create session
        let sess = Session::new(trans);

        // Create client
        let mut client = Self {
            sess,
            ctx: Mutex::new(0),
            client_index: 0,
            msg_id_map: HashMap::new(),
            msg_name_map: HashMap::new(),
        };

        // Init client
        client.init().await?;

        Ok(client)
    }

    pub async fn send_msg<T>(&self, msg: T) -> Result<u32>
    where
        T: Pack + MessageName + MessageId + MessageContext + MessageClientId,
    {
        let ctx = self.next_ctx().await;
        let msg_id = self.get_msg_id::<T>()?;
        let msg = msg
            .set_message_id(msg_id)
            .set_context(ctx)
            .set_client_index(self.client_index);

        self.sess.send_msg(Message::new(msg)).await?;

        Ok(ctx)
    }

    pub async fn recv_msg<T>(&mut self, ctx: u32) -> Result<T>
    where
        T: Pack + MessageName + MessageId + MessageContext,
    {
        let msg_id = self.get_msg_id::<T>()?;

        Ok(self.sess.recv_single_msg(ctx, msg_id).await?)
    }

    async fn init(&mut self) -> Result<()> {
        log::trace!("Init client");

        let ctx = self.next_ctx().await;

        // Send socket client init message
        let sock_clnt_create_msg = Message::new(VlApiSockclntCreateT {
            _vl_msg_id: VL_API_SOCK_CLNT_CREATE_MSG_ID,
            context: ctx,
            name: CLIENT_NAME.to_string(),
        });
        log::trace!("Send sockclnt create");
        self.sess.send_msg(sock_clnt_create_msg).await?;

        // Get vpp msg table
        // XXX: sockclnt reply context is 0
        log::trace!("Wait sockclnt create reply");
        let sock_clnt_rep_msg: VlApiSockclntCreateReplyT = self
            .sess
            .recv_single_msg(0, VL_API_SOCK_CLNT_CREATE_REP_MSG_ID)
            .await?;

        // Update client index
        log::trace!("Client index: {}", sock_clnt_rep_msg.index);
        self.client_index = sock_clnt_rep_msg.index;

        // Init hash
        self.init_msg_hash(&sock_clnt_rep_msg)?;

        Ok(())
    }

    fn init_msg_hash(&mut self, msg: &VlApiSockclntCreateReplyT) -> Result<()> {
        log::trace!("Init message hash");

        for entry in &msg.message_table {
            let id = entry.index;
            let last_underline_index = if let Some(pos) = entry.name.rfind("_") {
                pos
            } else {
                return Err(Error::internal("Missing '_' in table message"));
            };
            let name = entry.name[0..last_underline_index].to_string();
            let crc = entry.name[last_underline_index + 1..].to_string();
            let msg_entry = Arc::new(MessageEntry {
                id,
                name: name.clone(),
                crc,
            });

            self.msg_name_map.insert(name, msg_entry.clone());
            self.msg_id_map.insert(id, msg_entry);
        }

        Ok(())
    }

    fn get_msg_id<T>(&self) -> Result<u16>
    where
        T: MessageName,
    {
        let name = T::message_name();
        let info = self.msg_name_map.get(&name).ok_or(Error::argument(format!(
            "Message '{}' not found in vpp",
            name
        )))?;

        // TODO: Validate crc

        Ok(info.id)
    }

    async fn next_ctx(&self) -> u32 {
        let mut ctx = self.ctx.lock().await;
        *ctx += 1;

        *ctx
    }
}