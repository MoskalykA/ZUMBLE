use crate::client::Client;
use crate::error::MumbleError;
use crate::handler::Handler;
use crate::proto::mumble::UserState;
use crate::sync::RwLock;
use crate::ServerState;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
impl Handler for UserState {
    async fn handle(&self, state: Arc<RwLock<ServerState>>, client: Arc<RwLock<Client>>) -> Result<(), MumbleError> {
        let session_id = { client.read_err().await?.session_id };

        if self.get_session() != session_id {
            return Ok(());
        }

        {
            client.write_err().await?.update(self);
        }

        if self.has_channel_id() {
            let leave_channel_id = (state
                .read_err()
                .await?
                .set_client_channel(client.clone(), self.get_channel_id())
                .await)
                .unwrap_or_default();

            if let Some(leave_channel_id) = leave_channel_id {
                {
                    state.write_err().await?.channels.remove(&leave_channel_id);
                }
            }
        }

        let session_id = { client.read_err().await?.session_id };

        for channel_id in self.get_listening_channel_add() {
            {
                if let Some(channel) = state.read_err().await?.channels.get(channel_id) {
                    channel.write_err().await?.listeners.insert(session_id);
                }
            }
        }

        for channel_id in self.get_listening_channel_remove() {
            {
                if let Some(channel) = state.read_err().await?.channels.get(channel_id) {
                    channel.write_err().await?.listeners.remove(&session_id);
                }
            }
        }

        Ok(())
    }
}
