//! Async client transport wrapper
//!
//! Wraps a synchronous `ClientTransport` and provides async request/response
//! correlation via a background polling task and channels.

use lp_model::{ClientMessage, ServerMessage, TransportError};
use lp_shared::transport::ClientTransport;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

/// Async wrapper around a synchronous `ClientTransport`
///
/// Spawns a background task that polls the underlying transport for messages
/// and correlates responses to requests by ID. Provides async `send_request()`
/// method that waits for the corresponding response.
///
/// # Example
///
/// ```no_run
/// use lp_cli::client::{client_connect, async_transport::AsyncClientTransport, specifier::HostSpecifier};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = client_connect(HostSpecifier::Local)?;
/// let async_transport = AsyncClientTransport::new(transport);
///
/// let msg = ClientMessage { id: 1, msg: lp_model::ClientRequest::Ping };
/// let response = async_transport.send_request(msg).await?;
/// # Ok(())
/// # }
/// ```
pub struct AsyncClientTransport {
    /// Channel for sending requests to the polling task
    request_tx: Option<mpsc::UnboundedSender<(ClientMessage, oneshot::Sender<Result<ServerMessage, TransportError>>)>>,
    /// Channel for receiving critical transport errors
    error_rx: mpsc::Receiver<TransportError>,
    /// Handle to the background polling task
    poller_handle: Option<JoinHandle<()>>,
    /// Whether the transport has been closed
    closed: Arc<AtomicBool>,
}

impl AsyncClientTransport {
    /// Create a new async client transport
    ///
    /// Takes ownership of the underlying `ClientTransport` and spawns a background
    /// task that polls for messages and correlates responses to requests.
    ///
    /// # Arguments
    ///
    /// * `transport` - The underlying synchronous client transport
    ///
    /// # Returns
    ///
    /// * `Self` - The async transport wrapper
    pub fn new(mut transport: Box<dyn ClientTransport + Send>) -> Self {
        // Create channels for requests and errors
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (error_tx, error_rx) = mpsc::channel(100); // Bounded to prevent unbounded growth

        // Create closed flag
        let closed = Arc::new(AtomicBool::new(false));

        // Spawn background polling task
        let closed_clone = Arc::clone(&closed);
        let poller_handle = tokio::spawn(async move {
            // Track pending requests by ID
            let mut pending_requests: HashMap<u64, oneshot::Sender<Result<ServerMessage, TransportError>>> = HashMap::new();

            // Use request_rx as async receiver
            let mut request_rx: mpsc::UnboundedReceiver<(ClientMessage, oneshot::Sender<Result<ServerMessage, TransportError>>)> = request_rx;

            loop {
                // Check if closed
                if closed_clone.load(Ordering::Relaxed) {
                    break;
                }

                // Process incoming requests
                while let Ok((msg, response_tx)) = request_rx.try_recv() {
                    // Send the message via the underlying transport
                    if let Err(e) = transport.send(msg.clone()) {
                        // Send error to response channel
                        let _ = response_tx.send(Err(e.clone()));
                        // Also send to error channel
                        let _ = error_tx.send(e).await;
                        continue;
                    }

                    // Track the pending request
                    pending_requests.insert(msg.id, response_tx);
                }

                // Poll for incoming messages
                match transport.receive_all() {
                    Ok(messages) => {
                        for msg in messages {
                            // Find matching pending request
                            if let Some(response_tx) = pending_requests.remove(&msg.id) {
                                // Send response to waiting request
                                let _ = response_tx.send(Ok(msg));
                            } else {
                                // Orphaned response (no matching request)
                                // This can happen if the request timed out or was cancelled
                                // We'll just ignore it
                            }
                        }
                    }
                    Err(e) => {
                        // Transport error - send to error channel
                        let _ = error_tx.send(e.clone()).await;
                        // Also send error to all pending requests
                        for (_, response_tx) in pending_requests.drain() {
                            let _ = response_tx.send(Err(e.clone()));
                        }
                        // Exit polling loop on transport error
                        break;
                    }
                }

                // Yield to allow other tasks to run
                tokio::task::yield_now().await;

                // Small sleep to avoid busy-waiting
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }

            // Cleanup: send errors to any remaining pending requests
            for (_, response_tx) in pending_requests.drain() {
                let _ = response_tx.send(Err(TransportError::ConnectionLost));
            }
        });

        Self {
            request_tx: Some(request_tx),
            error_rx,
            poller_handle: Some(poller_handle),
            closed,
        }
    }

    /// Send a request and wait for the response
    ///
    /// Sends the request via the underlying transport and waits for the corresponding
    /// response. The response is matched to the request by ID.
    ///
    /// # Arguments
    ///
    /// * `msg` - The client message to send
    ///
    /// # Returns
    ///
    /// * `Ok(ServerMessage)` if the response was received
    /// * `Err(TransportError)` if sending failed or the transport was closed
    ///
    /// # Timeout
    ///
    /// This method does not have a built-in timeout. If you need timeout behavior,
    /// wrap the call with `tokio::time::timeout()`.
    pub async fn send_request(&self, msg: ClientMessage) -> Result<ServerMessage, TransportError> {
        // Check if closed
        if self.closed.load(Ordering::Relaxed) {
            return Err(TransportError::ConnectionLost);
        }

        // Create oneshot channel for response
        let (response_tx, response_rx) = oneshot::channel();

        // Send request to polling task
        match &self.request_tx {
            Some(tx) => tx
                .send((msg, response_tx))
                .map_err(|_| TransportError::ConnectionLost)?,
            None => return Err(TransportError::ConnectionLost),
        }

        // Wait for response
        response_rx
            .await
            .map_err(|_| TransportError::ConnectionLost)?
    }

    /// Get a reference to the error receiver channel
    ///
    /// Returns a reference to the channel that receives critical transport errors.
    /// Consumers can poll this channel to detect transport failures.
    ///
    /// # Returns
    ///
    /// * `&mpsc::Receiver<TransportError>` - Reference to the error channel
    pub fn error_rx(&self) -> &mpsc::Receiver<TransportError> {
        &self.error_rx
    }

    /// Close the transport and stop the polling task
    ///
    /// Sets the closed flag, closes the request channel (signaling the polling task
    /// to stop), and waits for the polling task to finish.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the transport was closed successfully
    /// * `Err(TransportError)` if closing failed
    pub async fn close(mut self) -> Result<(), TransportError> {
        // Check if already closed
        if self.closed.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Mark as closed
        self.closed.store(true, Ordering::Relaxed);

        // Close request channel (signals poller to stop)
        drop(self.request_tx.take());

        // Wait for polling task to finish
        if let Some(handle) = self.poller_handle.take() {
            handle.await.map_err(|_| {
                TransportError::Other("Polling task panicked".to_string())
            })?;
        }

        Ok(())
    }
}

impl Drop for AsyncClientTransport {
    fn drop(&mut self) {
        // If not already closed, try to close (best-effort)
        if !self.closed.load(Ordering::Relaxed) {
            // Mark as closed
            self.closed.store(true, Ordering::Relaxed);
            // Close request channel
            drop(self.request_tx.take());
            // Try to abort the polling task (can't await in Drop)
            if let Some(handle) = self.poller_handle.take() {
                handle.abort();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::local::create_local_transport_pair;
    use lp_shared::transport::ServerTransport;

    #[tokio::test]
    async fn test_send_request_waits_for_response() {
        // Create async local transport pair
        let (client_transport, mut server_transport) = create_local_transport_pair();

        // Wrap client transport in async wrapper
        let async_transport = AsyncClientTransport::new(Box::new(client_transport));

        // Spawn a task to send a response
        tokio::spawn(async move {
            // Wait a bit for the request to arrive
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // Receive the request
            if let Ok(Some(request)) = server_transport.receive() {
                // Send a response
                let response = ServerMessage {
                    id: request.id,
                    msg: lp_model::ServerResponse::ListAvailableProjects { projects: vec![] },
                };
                let _ = server_transport.send(response);
            }
        });

        // Send a request
        let request = ClientMessage {
            id: 1,
            msg: lp_model::ClientRequest::ListAvailableProjects,
        };
        let response = async_transport.send_request(request).await;

        // Verify we got a response
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.id, 1);
        assert!(matches!(response.msg, lp_model::ServerResponse::ListAvailableProjects { .. }));
    }

    #[tokio::test]
    async fn test_multiple_concurrent_requests() {
        // Create async local transport pair
        let (client_transport, mut server_transport) = create_local_transport_pair();

        // Wrap client transport in async wrapper
        let async_transport = AsyncClientTransport::new(Box::new(client_transport));

        // Wrap in Arc to share across tasks (AsyncClientTransport is Send but not Sync)
        // We'll use Arc<tokio::sync::Mutex> to allow sharing in async contexts
        use std::sync::Arc;
        let async_transport = Arc::new(tokio::sync::Mutex::new(async_transport));

        // Spawn a task to send responses
        tokio::spawn(async move {
            let mut counter = 0;
            loop {
                // Wait for requests
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                // Try to receive a request
                if let Ok(Some(request)) = server_transport.receive() {
                    counter += 1;
                    // Send response
                    let response = ServerMessage {
                        id: request.id,
                        msg: lp_model::ServerResponse::ListAvailableProjects { projects: vec![] },
                    };
                    let _ = server_transport.send(response);
                    if counter >= 3 {
                        break;
                    }
                }
            }
        });

        // Send multiple concurrent requests
        let mut handles = Vec::new();
        for i in 1..=3 {
            let async_transport = Arc::clone(&async_transport);
            let handle = tokio::spawn(async move {
                let request = ClientMessage {
                    id: i,
                    msg: lp_model::ClientRequest::ListAvailableProjects,
                };
                // Lock the mutex to get &self for send_request
                let transport = async_transport.lock().await;
                transport.send_request(request).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            let response = handle.await.unwrap();
            assert!(response.is_ok());
        }
    }

    #[tokio::test]
    async fn test_close_stops_polling_task() {
        // Create async local transport pair
        let (client_transport, _server_transport) = create_local_transport_pair();

        // Wrap client transport in async wrapper
        let async_transport = AsyncClientTransport::new(Box::new(client_transport));

        // Close the transport (this should stop the polling task)
        // Since close() takes ownership, we can't use the transport after this
        let close_result = async_transport.close().await;
        assert!(close_result.is_ok());
    }
}
