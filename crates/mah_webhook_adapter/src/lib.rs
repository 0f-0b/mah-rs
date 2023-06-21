#![forbid(unsafe_code)]

use std::convert::Infallible;
use std::net::SocketAddr;

use mah_core::event::MessageOrEvent;
use tokio::sync::mpsc;
use warp::{Filter as _, Rejection};

#[derive(Clone, Copy, Debug)]
pub struct WebhookAdapterEvents(());

impl WebhookAdapterEvents {
    pub fn new() -> Self {
        Self(())
    }

    pub fn listen(
        self,
        addr: impl Into<SocketAddr>,
        on_error: impl Fn(Rejection) + Clone + Send + Sync + 'static,
    ) -> Result<mpsc::UnboundedReceiver<MessageOrEvent>, warp::Error> {
        let addr = addr.into();
        let (tx, rx) = mpsc::unbounded_channel();
        let route = warp::body::content_length_limit(0x10000)
            .and(warp::body::json())
            .map({
                let tx = tx.clone();
                move |value| {
                    let _ = tx.send(value);
                    warp::http::StatusCode::NO_CONTENT
                }
            })
            .recover(move |err| {
                on_error(err);
                std::future::ready(Ok::<_, Infallible>(warp::http::StatusCode::BAD_REQUEST))
            });
        let (_, server) = warp::serve(route)
            .try_bind_with_graceful_shutdown(addr, async move { tx.closed().await })?;
        tokio::spawn(server);
        Ok(rx)
    }
}

impl Default for WebhookAdapterEvents {
    fn default() -> Self {
        Self::new()
    }
}
