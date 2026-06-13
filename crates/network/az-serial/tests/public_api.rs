use az_serial::{BaudRate, PortInfo, SerialConfig, SerialError, SerialPort};

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
fn serial_error_display() {
    let err = SerialError::PortNotFound("COM99".into());
    assert_eq!(err.to_string(), "port not found: COM99");

    let err = SerialError::Timeout(5000);
    assert_eq!(err.to_string(), "timeout after 5000ms");
}
