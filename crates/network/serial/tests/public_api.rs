use az_serial::config::{BaudRate, SerialConfig};
use az_serial::port::{PortInfo, SerialPort};

#[test]
fn open_empty_port_name_errors() {
    let config = SerialConfig::new(BaudRate::Baud9600);
    let result = SerialPort::open("", &config);
    assert!(result.is_err());
}

#[test]
fn open_zero_baud_rate_errors() {
    let config = SerialConfig::new(BaudRate::Baud0);
    let result = SerialPort::open("/dev/ttyUSB0", &config);
    assert!(result.is_err());
}

#[test]
fn write_to_closed_port_errors() {
    let config = SerialConfig::new(BaudRate::Baud9600);
    let mut port = SerialPort::open("/dev/ttyUSB0", &config).unwrap();
    port.close().unwrap();
    assert!(!port.is_open());
    let result = port.write(b"test");
    assert!(result.is_err());
}

#[test]
fn port_info_fields() {
    let info = PortInfo {
        port_name: "/dev/ttyUSB0".into(),
        description: "USB Serial".into(),
        vid: Some(0x1234),
        pid: Some(0x5678),
        serial_number: Some("ABC123".into()),
    };
    assert_eq!(info.port_name, "/dev/ttyUSB0");
    assert_eq!(info.vid, Some(0x1234));
}

#[test]
fn serial_errors_use_anyhow_messages() {
    let config = SerialConfig::new(BaudRate::Baud0);
    let error = SerialPort::open("/dev/ttyUSB0", &config).unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid config: baud rate cannot be zero"
    );
}
