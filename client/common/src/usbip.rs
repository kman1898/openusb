use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, info};

/// Represents a currently attached USB/IP device.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachedDevice {
    pub port: String,
    pub server: String,
    pub bus_id: String,
}

/// Attach a remote USB device via usbip.
pub async fn attach(server: &str, bus_id: &str) -> Result<()> {
    info!(server, bus_id, "Attaching USB/IP device");

    let output = Command::new("usbip")
        .args(["attach", "-r", server, "-b", bus_id])
        .output()
        .await
        .context("Failed to execute usbip attach")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("usbip attach failed: {}", stderr.trim());
    }

    info!(server, bus_id, "Device attached successfully");
    Ok(())
}

/// Detach a USB/IP device by finding it by bus_id.
pub async fn detach(bus_id: &str) -> Result<()> {
    // First find the port for this bus_id
    let attached = list_attached().await?;
    let device = attached
        .iter()
        .find(|d| d.bus_id == bus_id)
        .context("Device not found in attached list")?;

    detach_port(&device.port).await
}

/// Detach a USB/IP device by its local port number.
pub async fn detach_port(port: &str) -> Result<()> {
    info!(port, "Detaching USB/IP device");

    let output = Command::new("usbip")
        .args(["detach", "-p", port])
        .output()
        .await
        .context("Failed to execute usbip detach")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("usbip detach failed: {}", stderr.trim());
    }

    info!(port, "Device detached successfully");
    Ok(())
}

/// Detach all attached USB/IP devices.
pub async fn detach_all() -> Result<()> {
    let attached = list_attached().await?;
    for device in &attached {
        if let Err(e) = detach_port(&device.port).await {
            tracing::warn!(port = %device.port, "Failed to detach: {}", e);
        }
    }
    Ok(())
}

/// List currently attached USB/IP devices by parsing `usbip port`.
pub async fn list_attached() -> Result<Vec<AttachedDevice>> {
    let output = Command::new("usbip")
        .args(["port"])
        .output()
        .await
        .context("Failed to execute usbip port")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices = parse_usbip_port(&stdout);
    debug!("Found {} attached USB/IP devices", devices.len());
    Ok(devices)
}

fn parse_usbip_port(output: &str) -> Vec<AttachedDevice> {
    let mut devices = Vec::new();
    let mut current: Option<HashMap<&str, String>> = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Port") {
            // Save previous device
            if let Some(ref info) = current
                && let (Some(port), Some(server), Some(bus_id)) =
                    (info.get("port"), info.get("server"), info.get("bus_id"))
                {
                    devices.push(AttachedDevice {
                        port: port.clone(),
                        server: server.clone(),
                        bus_id: bus_id.clone(),
                    });
                }
            // Parse: "Port 00: <Import> ... at ..."
            let mut info = HashMap::new();
            if let Some(port_num) = trimmed
                .strip_prefix("Port ")
                .and_then(|s| s.split(':').next())
            {
                info.insert("port", port_num.trim().to_string());
            }
            current = Some(info);
        } else if let Some(ref mut info) = current {
            // Parse lines like: "  -> usbip://192.168.1.50:3240/1-1.2"
            if trimmed.contains("usbip://")
                && let Some(url) = trimmed.split("usbip://").nth(1)
                    && let Some((server_port, bus_id)) = url.split_once('/') {
                        let server = server_port
                            .rsplit_once(':')
                            .map(|(s, _)| s)
                            .unwrap_or(server_port);
                        info.insert("server", server.to_string());
                        info.insert("bus_id", bus_id.to_string());
                    }
        }
    }

    // Don't forget the last one
    if let Some(ref info) = current
        && let (Some(port), Some(server), Some(bus_id)) =
            (info.get("port"), info.get("server"), info.get("bus_id"))
        {
            devices.push(AttachedDevice {
                port: port.clone(),
                server: server.clone(),
                bus_id: bus_id.clone(),
            });
        }

    devices
}

/// Check if the usbip command is available.
pub async fn check_driver() -> Result<DriverStatus> {
    let output = Command::new("usbip")
        .arg("version")
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Ok(DriverStatus::Installed { version })
        }
        Ok(_) => Ok(DriverStatus::Error { message: "usbip command failed".into() }),
        Err(_) => Ok(DriverStatus::NotInstalled),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DriverStatus {
    Installed { version: String },
    NotInstalled,
    Error { message: String },
}
