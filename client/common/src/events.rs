use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum ClientEvent {
    DeviceAttached { server: String, bus_id: String },
    DeviceDetached { bus_id: String },
    ServerDiscovered { name: String, host: String },
    ServerLost { name: String },
    ReconnectAttempt { server: String, bus_id: String },
    ReconnectSuccess { server: String, bus_id: String },
}

pub struct EventBus {
    tx: broadcast::Sender<ClientEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self { tx }
    }

    pub fn emit(&self, event: ClientEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ClientEvent> {
        self.tx.subscribe()
    }
}
