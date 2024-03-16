use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
#[serde(tag = "kind", content = "data")]
pub enum DeviceStatusMQTTMessage {
    #[serde(rename = "0")]
    ReadBattery(Vec<ReadBatteryData>),
    #[serde(rename = "1")]
    ReadDeviceError(Vec<ReadDeviceErrorData>),
    #[serde(rename = "2")]
    ConnectDevice(Vec<ConnectDeviceData>),
    #[serde(rename = "3")]
    DisconnectDevice(Vec<DisconnectDeviceData>),
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct ReadBatteryData {
    id: u32,
    value: u8,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct ReadDeviceErrorData {
    id: u32,
    component: u8,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct ConnectDeviceData {
    id: u32,
    component: u8,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct DisconnectDeviceData {
    id: u32,
    component: u8,   
}

#[cfg(test)]
mod deserialize_tests {
    use super::{DeviceStatusMQTTMessage, ReadBatteryData, ReadDeviceErrorData, DisconnectDeviceData, ConnectDeviceData};

    #[test]
    fn deserialize_readbattery() {
        let input = r#"{
            "kind": "0",
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
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        let expected = DeviceStatusMQTTMessage::ReadBattery(vec![ReadBatteryData { id: 0, value: 30 }, ReadBatteryData { id: 1, value: 50 }, ReadBatteryData { id: 2, value: 50 }, ReadBatteryData { id: 3, value: 100 }, ReadBatteryData { id: 4, value: 0 }]);
        
        assert_eq!(result, expected);
    }
    
    #[test]
    fn deserialize_readdeviceerror() {
        let input = r#"{
            "kind": "1",
            "data": [
                {
                    "id": 0,
                    "component": 8 
                }
            ]
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        let expected = DeviceStatusMQTTMessage::ReadDeviceError(vec![ReadDeviceErrorData { id: 0, component: 8 }]);

        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_readconnectdevice() {
        let input = r#"{
            "kind": "2",
            "data": [
                {
                    "id": 0,
                    "component": 1 
                }
            ]
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        let expected = DeviceStatusMQTTMessage::ConnectDevice(vec![ConnectDeviceData { id: 0, component: 1 }]);
        
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_readdisconnectdevice() {
        let input = r#"{
            "kind": "3",
            "data": [
                {
                    "id": 0,
                    "component": 1 
                }
            ]
        }"#;

        let result: DeviceStatusMQTTMessage = serde_json::from_str(input).unwrap();
        let expected = DeviceStatusMQTTMessage::DisconnectDevice(vec![DisconnectDeviceData { id: 0, component: 1 }]);
        
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod serialize_tests {
    use super::{DeviceStatusMQTTMessage, ReadBatteryData, ReadDeviceErrorData, DisconnectDeviceData, ConnectDeviceData};

    #[test]
    fn serialize_readbattery() {
        let input = DeviceStatusMQTTMessage::ReadBattery(vec![ReadBatteryData { id: 0, value: 30 }, ReadBatteryData { id: 1, value: 50 }, ReadBatteryData { id: 2, value: 50 }, ReadBatteryData { id: 3, value: 100 }, ReadBatteryData { id: 4, value: 0 }]);
        let result = serde_json::to_string(&input).unwrap().parse::<serde_json::Value>().unwrap();
        let expected = r#"{
            "kind": "0",
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
        }"#.parse::<serde_json::Value>().unwrap();

        assert_eq!(result, expected);
    }
    
    #[test]
    fn serialize_readdeviceerror() {
        let input = DeviceStatusMQTTMessage::ReadDeviceError(vec![ReadDeviceErrorData { id: 0, component: 8 }]);
        let result = serde_json::to_string(&input).unwrap().parse::<serde_json::Value>().unwrap();
        let expected = r#"{
            "kind": "1",
            "data": [
                {
                    "id": 0,
                    "component": 8 
                }
            ]
        }"#.parse::<serde_json::Value>().unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_readconnectdevice() {
        let input = DeviceStatusMQTTMessage::ConnectDevice(vec![ConnectDeviceData { id: 0, component: 1 }]);
        let result = serde_json::to_string(&input).unwrap().parse::<serde_json::Value>().unwrap();

        let expected = r#"{
            "kind": "2",
            "data": [
                {
                    "id": 0,
                    "component": 1 
                }
            ]
        }"#.parse::<serde_json::Value>().unwrap();        
        
        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_readdisconnectdevice() {
        let input = DeviceStatusMQTTMessage::DisconnectDevice(vec![DisconnectDeviceData { id: 0, component: 1 }]);
        let result = serde_json::to_string(&input).unwrap().parse::<serde_json::Value>().unwrap();

        let expected = r#"{
            "kind": "3",
            "data": [
                {
                    "id": 0,
                    "component": 1 
                }
            ]
        }"#.parse::<serde_json::Value>().unwrap();

        assert_eq!(result, expected);
    }
}