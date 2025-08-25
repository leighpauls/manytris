use std::{error::Error, fmt::Display, net::SocketAddr};

use anyhow::{Context, Result};
use futures::{future, StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    sync::oneshot::{self, Sender},
    task::JoinHandle,
};
use tokio_stream::wrappers::TcpListenerStream;

pub struct Forwarder {
    listener_port: u16,
    join_handle: JoinHandle<()>,
    exit_sender: Sender<()>,
}

impl Forwarder {
    pub fn listener_port(&self) -> u16 {
        self.listener_port
    }

    pub async fn exit_join(self) -> Result<()> {
        self.exit_sender.send(()).map_err(|_| ExitSignalError)?;
        self.join_handle.await?;
        Ok(())
    }
}

#[derive(Debug)]
struct ExitSignalError;

impl Error for ExitSignalError {}

impl Display for ExitSignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{self:?}")
    }
}

pub async fn bind_ports(
    pods: Api<Pod>,
    pod_name: String,
    pod_ports: &[u16],
) -> Result<Vec<Forwarder>> {
    let (ok, err): (Vec<_>, Vec<_>) = future::join_all(
        pod_ports
            .iter()
            .map(|pod_port| bind_port(pods.clone(), pod_name.clone(), *pod_port)),
    )
    .await
    .into_iter()
    .partition(|r| r.is_ok());

    let Some(Err(first_err)) = err.into_iter().next() else {
        return Ok(ok.into_iter().collect::<Result<Vec<_>>>().unwrap());
    };

    // Failure, cancel any existing forwarders
    future::join_all(ok.into_iter().filter_map(|r| Some(r.ok()?.exit_join())))
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()
        .context("Err while stopping forwarders due to partial error.")?;

    return Err(first_err);
}

pub async fn bind_port(pods: Api<Pod>, pod_name: String, pod_port: u16) -> Result<Forwarder> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await?;

    let listener_port = listener.local_addr()?.port();

    let (exit_sender, rx) = oneshot::channel::<()>();

    let join_handle = tokio::spawn(async move {
        let server = TcpListenerStream::new(listener)
            .take_until(rx)
            .try_for_each(|client_conn| async {
                let pods = pods.clone();
                let pod_name = pod_name.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        forward_connection(&pods, &pod_name, pod_port, client_conn).await
                    {
                        eprintln!("Failed to forward connection: {e}");
                    }
                });

                Ok(())
            });

        if let Err(e) = server.await {
            eprintln!("Error exiting listener server: {e}");
        } else {
            println!("Exiting cleanly");
        }
    });

    Ok(Forwarder {
        listener_port,
        join_handle,
        exit_sender,
    })
}

async fn forward_connection(
    pods: &Api<Pod>,
    pod_name: &str,
    port: u16,
    mut client_conn: impl AsyncRead + AsyncWrite + Unpin,
) -> anyhow::Result<()> {
    println!("Creating forwarder {pod_name} {port}");
    let mut forwarder = pods.portforward(pod_name, &[port]).await?;
    let mut upstream_conn = forwarder
        .take_stream(port)
        .context("port not found in forwarder")?;

    println!("Starting copy stream");
    tokio::io::copy_bidirectional(&mut client_conn, &mut upstream_conn).await?;
    println!("Left copy stream");
    drop(upstream_conn);
    println!("Joining forwarder");
    forwarder.join().await?;
    println!("connection closed");
    Ok(())
}
