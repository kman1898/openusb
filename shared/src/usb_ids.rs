use std::collections::HashMap;

/// Parser for the USB ID database (usb.ids file).
/// Maps vendor IDs to vendor names and product IDs to product names.
pub struct UsbIdDatabase {
    vendors: HashMap<u16, Vendor>,
}

struct Vendor {
    name: String,
    products: HashMap<u16, String>,
}

impl UsbIdDatabase {
    /// Parse a usb.ids file from its contents.
    pub fn parse(contents: &str) -> Self {
        let mut vendors = HashMap::new();
        let mut current_vendor: Option<(u16, Vendor)> = None;

        for line in contents.lines() {
            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // Product line (starts with a tab)
            if line.starts_with('\t') && !line.starts_with("\t\t") {
                let trimmed = line.trim();
                if let Some((id_str, name)) = trimmed.split_once("  ")
                    && let Ok(id) = u16::from_str_radix(id_str.trim(), 16)
                    && let Some((_, ref mut vendor)) = current_vendor
                {
                    vendor.products.insert(id, name.trim().to_string());
                }
                continue;
            }

            // Vendor line (starts with hex digit, no leading whitespace)
            if !line.starts_with('\t') {
                // Save previous vendor
                if let Some((vid, vendor)) = current_vendor.take() {
                    vendors.insert(vid, vendor);
                }

                if let Some((id_str, name)) = line.split_once("  ")
                    && let Ok(id) = u16::from_str_radix(id_str.trim(), 16)
                {
                    current_vendor = Some((
                        id,
                        Vendor {
                            name: name.trim().to_string(),
                            products: HashMap::new(),
                        },
                    ));
                }
            }
        }

        // Don't forget the last vendor
        if let Some((vid, vendor)) = current_vendor {
            vendors.insert(vid, vendor);
        }

        Self { vendors }
    }

    /// Look up a vendor name by ID.
    pub fn vendor_name(&self, vendor_id: u16) -> Option<&str> {
        self.vendors.get(&vendor_id).map(|v| v.name.as_str())
    }

    /// Look up a product name by vendor and product ID.
    pub fn product_name(&self, vendor_id: u16, product_id: u16) -> Option<&str> {
        self.vendors
            .get(&vendor_id)
            .and_then(|v| v.products.get(&product_id))
            .map(|s| s.as_str())
    }
}
