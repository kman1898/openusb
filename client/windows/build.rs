fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "OpenUSB");
        res.set("FileDescription", "OpenUSB Client - USB device sharing");
        res.set("CompanyName", "OpenUSB");
        // Icon will be set when we have an .ico file:
        // res.set_icon("assets/openusb.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
