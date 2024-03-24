use serde::{Deserialize, Serialize};

use crate::backend_core::features::fire_alert_feature::models::FireStatus;

type Token = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(test, derive(std::cmp::PartialEq))]
#[serde(tag = "kind", content = "payload")]
pub enum FireMQTTMessage {
    #[serde(rename = "0")]
    Safe {
        token: Token,
        #[serde(rename = "fire")]
        fire_data: Vec<SensorData>,
        smoke: Vec<SensorData>,
        co: Vec<SensorData>,
        heat: Vec<SensorData>,
        #[serde(rename = "fire-button")]
        fire_button: Vec<SensorData>,
    },
    #[serde(rename = "1")]
    Unsafe {
        token: Token,
        #[serde(rename = "fire")]
        fire_data: Vec<SensorData>,
        smoke: Vec<SensorData>,
        co: Vec<SensorData>,
        heat: Vec<SensorData>,
        #[serde(rename = "fire-button")]
        fire_button: Vec<SensorData>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(test, derive(std::cmp::PartialEq))]
pub struct SensorData {
    pub id: u32,
    pub component: String,
    pub value: u32,
    pub alert: FireStatus,
}

#[cfg(test)]
mod deserialize_tests {
    use super::{FireMQTTMessage, FireStatus, SensorData, Token};

    #[test]
    fn deserialize_safe_data() {
        let input = r#"{
            "kind": "0",
            "payload": {
                "token": "abcd",
                "fire": [
                    {
                        "id": 0,
                        "component": "FIRE_0",
                        "value": 460,
                        "alert": "SAFE"
                    }
                ],
                "smoke": [
                    {
                        "id": 0,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    },
                    {
                        "id": 0,
                        "component": "SMOKE_1",
                        "value": 240,
                        "alert": "SAFE"
                    },
                    {
                        "id": 1,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    },
                    {
                        "id": 2,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    },
                    {
                        "id": 0,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    }
                ],
                "co": [
                    {
                        "id": 0,
                        "component": "CO_0",
                        "value": 460,
                        "alert": "SAFE"
                    },
                    {
                        "id": 1,
                        "component": "CO_0",
                        "value": 460,
                        "alert": "SAFE"
                    }
                ],
                "heat": [
                    {
                        "id": 2,
                        "component": "HEAT_0",
                        "value": 460,
                        "alert": "SAFE"
                    },
                    {
                        "id": 3,
                        "component": "HEAT_0",
                        "value": 460,
                        "alert": "SAFE"
                    }
                ],
                "fire-button": [
                    {
                        "id": 1,
                        "component": "BUTTON_0",
                        "value": 1,
                        "alert": "SAFE"
                    }
                ]
            }
        }"#;

        let result: FireMQTTMessage = serde_json::from_str(input).unwrap();

        let expected = FireMQTTMessage::Safe {
            token: Token::from("abcd"),
            fire_data: vec![SensorData {
                id: 0,
                component: "FIRE_0".to_string(),
                value: 460,
                alert: FireStatus::SAFE,
            }],
            smoke: vec![
                SensorData {
                    id: 0,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: "SMOKE_1".to_string(),
                    value: 240,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 2,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
            ],
            co: vec![
                SensorData {
                    id: 0,
                    component: "CO_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: "CO_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            heat: vec![
                SensorData {
                    id: 2,
                    component: "HEAT_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 3,
                    component: "HEAT_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            fire_button: vec![SensorData {
                id: 1,
                component: "BUTTON_0".to_string(),
                value: 1,
                alert: FireStatus::SAFE,
            }],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_unsafe_data() {
        let input = r#"{
            "kind": "1",
            "payload": {
                "token": "efgh",
                "fire": [
                    {
                        "id": 0,
                        "component": "FIRE_0",
                        "value": 460,
                        "alert": "UNSAFE"
                    }
                ],
                "smoke": [],
                "co": [],
                "heat": [],
                "fire-button": []
            }
        }"#;

        let result: FireMQTTMessage = serde_json::from_str(input).unwrap();

        let expected = FireMQTTMessage::Unsafe {
            token: Token::from("efgh"),
            fire_data: vec![SensorData {
                id: 0,
                component: "FIRE_0".to_string(),
                value: 460,
                alert: FireStatus::UNSAFE,
            }],
            smoke: vec![],
            co: vec![],
            heat: vec![],
            fire_button: vec![],
        };

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod serialize_tests {

    use serde_json::json;

    use super::{FireMQTTMessage, FireStatus, SensorData, Token};

    #[test]
    fn serialize_safe_data() {
        let input = FireMQTTMessage::Safe {
            token: Token::from("abcd"),
            fire_data: vec![SensorData {
                id: 0,
                component: "FIRE_0".to_string(),
                value: 460,
                alert: FireStatus::SAFE,
            }],
            smoke: vec![
                SensorData {
                    id: 0,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: "SMOKE_1".to_string(),
                    value: 240,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 2,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: "SMOKE_0".to_string(),
                    value: 120,
                    alert: FireStatus::SAFE,
                },
            ],
            co: vec![
                SensorData {
                    id: 0,
                    component: "CO_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: "CO_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            heat: vec![
                SensorData {
                    id: 2,
                    component: "HEAT_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 3,
                    component: "HEAT_0".to_string(),
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            fire_button: vec![SensorData {
                id: 1,
                component: "BUTTON_0".to_string(),
                value: 1,
                alert: FireStatus::SAFE,
            }],
        };

        let expected = json!({
            "kind": "0",
            "payload": {
                "token": "abcd",
                "fire": [
                    {
                        "id": 0,
                        "component": "FIRE_0",
                        "value": 460,
                        "alert": "SAFE"
                    }
                ],
                "smoke": [
                    {
                        "id": 0,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    },
                    {
                        "id": 0,
                        "component": "SMOKE_1",
                        "value": 240,
                        "alert": "SAFE"
                    },
                    {
                        "id": 1,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    },
                    {
                        "id": 2,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    },
                    {
                        "id": 0,
                        "component": "SMOKE_0",
                        "value": 120,
                        "alert": "SAFE"
                    }
                ],
                "co": [
                    {
                        "id": 0,
                        "component": "CO_0",
                        "value": 460,
                        "alert": "SAFE"
                    },
                    {
                        "id": 1,
                        "component": "CO_0",
                        "value": 460,
                        "alert": "SAFE"
                    }
                ],
                "heat": [
                    {
                        "id": 2,
                        "component": "HEAT_0",
                        "value": 460,
                        "alert": "SAFE"
                    },
                    {
                        "id": 3,
                        "component": "HEAT_0",
                        "value": 460,
                        "alert": "SAFE"
                    }
                ],
                "fire-button": [
                    {
                        "id": 1,
                        "component": "BUTTON_0",
                        "value": 1,
                        "alert": "SAFE"
                    }
                ]
            }
        });

        let result = serde_json::to_string_pretty(&input).unwrap();

        assert_eq!(result, serde_json::to_string_pretty(&expected).unwrap());
    }

    #[test]
    fn serialize_unsafe_data() {
        let input = FireMQTTMessage::Unsafe {
            token: Token::from("efgh"),
            fire_data: vec![SensorData {
                id: 0,
                component: "FIRE_0".to_string(),
                value: 460,
                alert: FireStatus::UNSAFE,
            }],
            smoke: vec![],
            co: vec![],
            heat: vec![],
            fire_button: vec![],
        };

        let expected = json!({
            "kind": "1",
            "payload": {
                "token": "efgh",
                "fire": [
                    {
                        "id": 0,
                        "component": "FIRE_0",
                        "value": 460,
                        "alert": "UNSAFE"
                    }
                ],
                "smoke": [],
                "co": [],
                "heat": [],
                "fire-button": []
            }
        });

        let result = serde_json::to_string_pretty(&input).unwrap();

        assert_eq!(result, serde_json::to_string_pretty(&expected).unwrap());
    }
}
