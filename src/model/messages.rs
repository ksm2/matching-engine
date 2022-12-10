use std::fmt::Debug;
use std::ops::Deref;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct MessageChannel<Req, Res> {
    receiver: oneshot::Receiver<Res>,
    port: MessagePort<Req, Res>,
}

impl<Req, Res> MessageChannel<Req, Res>
where
    Req: Send + Sync + Debug + 'static,
    Res: Send + Sync + Debug + 'static,
{
    pub fn new(req: Req) -> Self {
        // Create a one shot channel to reply
        let (sender, receiver) = oneshot::channel();
        let port = MessagePort { sender, req };
        Self { receiver, port }
    }

    pub async fn send_to(
        self,
        target: &mpsc::Sender<MessagePort<Req, Res>>,
    ) -> anyhow::Result<Res> {
        target.send(self.port).await?;
        let response = self.receiver.await?;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct MessagePort<Req, Res> {
    pub req: Req,
    sender: oneshot::Sender<Res>,
}

impl<Req, Res> MessagePort<Req, Res> {
    pub fn reply(self, res: Res) -> Result<(), Res> {
        self.sender.send(res)
    }
}

impl<Req, Res> Deref for MessagePort<Req, Res> {
    type Target = Req;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}
