use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::Eq, std::cmp::PartialEq, Debug))]
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
#[cfg_attr(test, derive(std::cmp::Eq, std::cmp::PartialEq, Debug))]
pub struct ReadBatteryData {
    id: u32,
    value: u8,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::Eq, std::cmp::PartialEq, Debug))]
pub struct ReadDeviceErrorData {
    id: u32,
    component: u8,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::Eq, std::cmp::PartialEq, Debug))]
pub struct ConnectDeviceData {
    id: u32,
    component: u8,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::Eq, std::cmp::PartialEq, Debug))]
pub struct DisconnectDeviceData {
    id: u32,
    component: u8,   
}

#[cfg(test)]
mod deserialize_tests {
    use super::{DeviceStatusMQTTMessage, ReadBatteryData};

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
}