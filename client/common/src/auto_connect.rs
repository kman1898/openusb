use crate::config::AutoUseRule;
use openusb_shared::device::UsbDevice;

/// Check if a device matches any auto-use rule.
pub fn should_auto_use(device: &UsbDevice, rules: &[AutoUseRule], server_name: &str) -> bool {
    let vid = format!("{:04x}", device.vendor_id);
    let pid = format!("{:04x}", device.product_id);

    for rule in rules {
        match rule {
            AutoUseRule::All => return true,
            AutoUseRule::Server { server } => {
                if server == server_name {
                    return true;
                }
            }
            AutoUseRule::VendorId { vendor_id } => {
                if vendor_id == &vid {
                    return true;
                }
            }
            AutoUseRule::Device {
                vendor_id,
                product_id,
            } => {
                if vendor_id == &vid && product_id == &pid {
                    return true;
                }
            }
        }
    }
    false
}
