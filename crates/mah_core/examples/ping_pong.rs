use std::net::Ipv4Addr;
use std::sync::Arc;

use anyhow::bail;
use mah_core::adapter::MahSession;
use mah_core::event::MessageOrEvent;
use mah_core::make_message;
use mah_core::message::{AnyMessage as _, IncomingMessageNode, Message};
use mah_http_adapter::HttpAdapter;
use mah_webhook_adapter::WebhookAdapterEvents;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use trim_in_place::TrimInPlace as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if !(3..=4).contains(&args.len()) {
        bail!(
            "usage: {} <webhook-port> <http-endpoint> [http-verify-key]",
            args[0]
        );
    }
    let port = args[1].parse()?;
    let endpoint = args[2].parse()?;
    let verify_key = args.get(3);
    let tracker = TaskTracker::new();
    let token = CancellationToken::new();
    tokio::spawn({
        let tracker = tracker.clone();
        let token = token.clone();
        async move {
            tokio::signal::ctrl_c().await.unwrap();
            tracker.close();
            token.cancel();
        }
    });
    let mah = HttpAdapter::new(endpoint, verify_key.cloned());
    let session = Arc::new(mah.verify().await?);
    let mut events = WebhookAdapterEvents::new().listen((Ipv4Addr::LOCALHOST, port), |err| {
        eprintln!("{err:?}");
    })?;
    while let Some(event) = loop {
        tokio::select! {
            event = events.recv() => break event,
            _ = token.cancelled() => {
                events.close();
                continue;
            }
        }
    } {
        let session = session.clone();
        tracker.spawn(async move {
            if let Err(err) = handle_event(session.as_ref(), event).await {
                eprintln!("{err}");
            }
        });
    }
    tracker.wait().await;
    Ok(())
}

async fn handle_event<S: MahSession + ?Sized + 'static>(
    session: &S,
    event: MessageOrEvent,
) -> anyhow::Result<()> {
    if let MessageOrEvent::Message(Message::Friend(message)) = &event {
        let text = get_text(message.nodes());
        if text == "ping" {
            println!("pong {:?}", message.context().handle());
            message
                .sender
                .handle()
                .send_message(session, &make_message!["pong"].quote(message.handle()))
                .await?;
        }
    }
    Ok(())
}

fn get_text(nodes: &[IncomingMessageNode]) -> String {
    let mut text = nodes
        .iter()
        .filter_map(|node| {
            if let IncomingMessageNode::Plain(node) = node {
                Some(node.text.as_ref())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    text.trim_in_place();
    text
}
