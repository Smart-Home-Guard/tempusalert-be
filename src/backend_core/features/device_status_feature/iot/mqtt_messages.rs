use serde::{Deserialize, Serialize};

type Token = String;

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
#[serde(tag = "kind", content = "payload")]
pub enum DeviceStatusMQTTMessage {
    #[serde(rename = "0")]
    ReadBattery {
        token: Token,
        data: Vec<ReadBatteryData>,
    },
    #[serde(rename = "1")]
    ReadDeviceError {
        token: Token,
        data: Vec<ReadDeviceErrorData>,
    },
    #[serde(rename = "2")]
    ConnectDevice {
        token: Token,
        data: Vec<ConnectDeviceData>,
    },
    #[serde(rename = "3")]
    DisconnectDevice {
        token: Token,
        data: Vec<DisconnectDeviceData>,
    },
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct ReadBatteryData {
    pub id: u32,
    pub value: u32,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct ReadDeviceErrorData {
    pub id: u32,
    pub component: u32,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct ConnectDeviceData {
    pub id: u32,
    pub component: u32,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct DisconnectDeviceData {
    pub id: u32,
    pub component: u32,
}

#[cfg(test)]
mod deserialize_tests {
    use super::{
        ConnectDeviceData, DeviceStatusMQTTMessage, DisconnectDeviceData, ReadBatteryData,
        ReadDeviceErrorData, Token,
    };

    #[test]
    fn deserialize_readbattery() {
        let input = r#"{
            "kind": "0",
            "payload": {
                "token": "abcd",
                "data": [
                    {
                        "id": 0,
                        "value": 30
                    },
                    {
                        "id": 1,
                        "value": 50
                    },
                    {
                        "id": 2,
                        "value": 50
                    },
                    {
                        "id": 3,
                        "value": 100
                    },
                    {
                        "id": 4,
                        "value": 0
                    }
                ]
            }
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        println!("{:?}", result);
        let expected = DeviceStatusMQTTMessage::ReadBattery {
            token: Token::from("abcd"),
            data: vec![
                ReadBatteryData { id: 0, value: 30 },
                ReadBatteryData { id: 1, value: 50 },
                ReadBatteryData { id: 2, value: 50 },
                ReadBatteryData { id: 3, value: 100 },
                ReadBatteryData { id: 4, value: 0 },
            ],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_readdeviceerror() {
        let input = r#"{
            "kind": "1",
            "payload": {
                "token": "abcd",
                "data": [
                    {
                        "id": 0,
                        "component": 8 
                    }
                ]
            }
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        let expected = DeviceStatusMQTTMessage::ReadDeviceError {
            token: Token::from("abcd"),
            data: vec![ReadDeviceErrorData {
                id: 0,
                component: 8,
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_readconnectdevice() {
        let input = r#"{
            "kind": "2",
            "payload": {
                "token": "abcd",
                "data":  [
                    {
                        "id": 0,
                        "component": 1 
                    }
                ]
            }
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        let expected = DeviceStatusMQTTMessage::ConnectDevice {
            token: Token::from("abcd"),
            data: vec![ConnectDeviceData {
                id: 0,
                component: 1,
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_readdisconnectdevice() {
        let input = r#"{
            "kind": "3",
            "payload": {
                "token": "abcd",
                "data": [
                    {
                        "id": 0,
                        "component": 1 
                    }
                ]
            }
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        let expected = DeviceStatusMQTTMessage::DisconnectDevice {
            token: Token::from("abcd"),
            data: vec![DisconnectDeviceData {
                id: 0,
                component: 1,
            }],
        };

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod serialize_tests {
    use super::{
        ConnectDeviceData, DeviceStatusMQTTMessage, DisconnectDeviceData, ReadBatteryData,
        ReadDeviceErrorData, Token,
    };

    #[test]
    fn serialize_readbattery() {
        let input = DeviceStatusMQTTMessage::ReadBattery {
            token: Token::from("abcd"),
            data: vec![
                ReadBatteryData { id: 0, value: 30 },
                ReadBatteryData { id: 1, value: 50 },
                ReadBatteryData { id: 2, value: 50 },
                ReadBatteryData { id: 3, value: 100 },
                ReadBatteryData { id: 4, value: 0 },
            ],
        };
        let result = serde_json::to_string(&input)
            .unwrap()
            .parse::<serde_json::Value>()
            .unwrap();
        let expected = r#"{
            "kind": "0",
            "payload": {
                "token": "abcd",
                "data": [
                    {
                        "id": 0,
                        "value": 30
                    },
                    {
                        "id": 1,
                        "value": 50
                    },
                    {
                        "id": 2,
                        "value": 50
                    },
                    {
                        "id": 3,
                        "value": 100
                    },
                    {
                        "id": 4,
                        "value": 0
                    }
                ]
            }
        }"#
        .parse::<serde_json::Value>()
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_readdeviceerror() {
        let input = DeviceStatusMQTTMessage::ReadDeviceError {
            token: Token::from("abcd"),
            data: vec![ReadDeviceErrorData {
                id: 0,
                component: 8,
            }],
        };
        let result = serde_json::to_string(&input)
            .unwrap()
            .parse::<serde_json::Value>()
            .unwrap();
        let expected = r#"{
            "kind": "1",
            "payload": {
                "token": "abcd",
                "data": [
                    {
                        "id": 0,
                        "component": 8 
                    }
                ]
            }
        }"#
        .parse::<serde_json::Value>()
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_readconnectdevice() {
        let input = DeviceStatusMQTTMessage::ConnectDevice {
            token: Token::from("abcd"),
            data: vec![ConnectDeviceData {
                id: 0,
                component: 1,
            }],
        };
        let result = serde_json::to_string(&input)
            .unwrap()
            .parse::<serde_json::Value>()
            .unwrap();

        let expected = r#"{
            "kind": "2",
            "payload": {
                "token": "abcd",
                "data": [
                    {
                        "id": 0,
                        "component": 1 
                    }
                ]
            }
        }"#
        .parse::<serde_json::Value>()
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_readdisconnectdevice() {
        let input = DeviceStatusMQTTMessage::DisconnectDevice {
            token: Token::from("abcd"),
            data: vec![DisconnectDeviceData {
                id: 0,
                component: 1,
            }],
        };
        let result = serde_json::to_string(&input)
            .unwrap()
            .parse::<serde_json::Value>()
            .unwrap();

        let expected = r#"{
            "kind": "3",
            "payload": {
                "token": "abcd",
                "data": [
                    {
                        "id": 0,
                        "component": 1 
                    }
                ]
            }
        }"#
        .parse::<serde_json::Value>()
        .unwrap();

        assert_eq!(result, expected);
    }
}
