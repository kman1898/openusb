Name:           openusb-server
Version:        0.1.0
Release:        1%{?dist}
Summary:        OpenUSB Server - Share USB devices over the network
License:        MIT
URL:            https://github.com/kman1898/openusb

%description
OpenUSB is an open-source USB-over-IP solution that allows sharing
USB devices connected to one machine with other machines on the network.
This package contains the server daemon (openusbd) and web dashboard.

%install
mkdir -p %{buildroot}/usr/local/bin
mkdir -p %{buildroot}/usr/share/openusb/web
mkdir -p %{buildroot}/etc/systemd/system
mkdir -p %{buildroot}/etc/openusb
mkdir -p %{buildroot}/var/lib/openusb
mkdir -p %{buildroot}/var/log/openusb

install -m 755 openusbd %{buildroot}/usr/local/bin/openusbd
install -m 755 openusb %{buildroot}/usr/local/bin/openusb 2>/dev/null || true
install -m 644 openusb.toml.example %{buildroot}/usr/share/openusb/openusb.toml.example
install -m 644 openusbd.service %{buildroot}/etc/systemd/system/openusbd.service

# Web dashboard files
cp -r web/* %{buildroot}/usr/share/openusb/web/ 2>/dev/null || true

%post
if [ ! -f /etc/openusb/openusb.toml ]; then
    cp /usr/share/openusb/openusb.toml.example /etc/openusb/openusb.toml
fi
modprobe usbip-core 2>/dev/null || true
modprobe usbip-host 2>/dev/null || true
systemctl daemon-reload
systemctl enable openusbd.service
systemctl start openusbd.service || true

%preun
systemctl stop openusbd.service 2>/dev/null || true
systemctl disable openusbd.service 2>/dev/null || true

%files
/usr/local/bin/openusbd
/usr/local/bin/openusb
/usr/share/openusb/
/etc/systemd/system/openusbd.service
%dir /etc/openusb
%dir /var/lib/openusb
%dir /var/log/openusb
