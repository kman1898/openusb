use openusb_shared::device::UsbDevice;
use openusb_shared::protocol::ServerInfo;

/// HTTP client for communicating with an OpenUSB server's REST API.
pub struct ServerClient {
    base_url: String,
    http: reqwest::Client,
}

impl ServerClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: format!("http://{}:{}/api/v1", host, port),
            http: reqwest::Client::new(),
        }
    }

    pub async fn server_info(&self) -> anyhow::Result<ServerInfo> {
        let resp = self
            .http
            .get(format!("{}/server/info", self.base_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn list_devices(&self) -> anyhow::Result<Vec<UsbDevice>> {
        let resp = self
            .http
            .get(format!("{}/devices", self.base_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn share_device(&self, bus_id: &str) -> anyhow::Result<()> {
        self.http
            .post(format!("{}/devices/{}/share", self.base_url, bus_id))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn unshare_device(&self, bus_id: &str) -> anyhow::Result<()> {
        self.http
            .post(format!("{}/devices/{}/unshare", self.base_url, bus_id))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn set_nickname(&self, bus_id: &str, nickname: &str) -> anyhow::Result<()> {
        self.http
            .put(format!("{}/devices/{}/nickname", self.base_url, bus_id))
            .json(&serde_json::json!({ "nickname": nickname }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
