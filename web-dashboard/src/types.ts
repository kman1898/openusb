export interface UsbDevice {
  id: string;
  bus_id: string;
  vendor_id: number;
  product_id: number;
  device_class: number;
  device_subclass: number;
  device_protocol: number;
  vendor_name: string | null;
  product_name: string | null;
  nickname: string | null;
  serial: string | null;
  num_configurations: number;
  speed: UsbSpeed;
  state: DeviceState;
}

export type UsbSpeed = "low" | "full" | "high" | "super" | "superplus" | "unknown";

export type DeviceState =
  | { status: "not_shared" }
  | { status: "available" }
  | { status: "in_use"; client_ip: string; client_name: string | null; since: string };

export interface ServerInfo {
  name: string;
  hostname: string;
  version: string;
  api_port: number;
  usbip_port: number;
  device_count: number;
  client_count: number;
  uptime_seconds: number;
  tls_enabled: boolean;
  auth_required: boolean;
}

export type ServerEvent =
  | { type: "device_attached"; device: UsbDevice }
  | { type: "device_detached"; bus_id: string }
  | { type: "device_shared"; bus_id: string }
  | { type: "device_unshared"; bus_id: string }
  | { type: "client_connected"; client_ip: string; client_name: string | null }
  | { type: "client_disconnected"; client_ip: string }
  | { type: "device_in_use"; bus_id: string; client_ip: string }
  | { type: "device_released"; bus_id: string };

export interface ClientStatus {
  installed: boolean;
  version?: string;
  driver_status?: DriverStatus;
}

export type DriverStatus =
  | { status: "installed"; version: string }
  | { status: "not_installed" }
  | { status: "error"; message: string };

export interface AutoUseRule {
  type: "device" | "vendor_id" | "server" | "all";
  vendor_id?: string;
  product_id?: string;
  server?: string;
}
