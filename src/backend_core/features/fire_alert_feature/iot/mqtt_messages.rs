use serde::{Deserialize, Serialize};

use crate::backend_core::features::fire_alert_feature::models::FireStatus;

type Token = String;

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
#[serde(tag = "kind", content = "payload")]
pub enum FireMQTTMessage {
    #[serde(rename = "0")]
    Periodic {
        token: Token,
        #[serde(rename = "fire")]
        fire_data: Vec<SensorData>,
        smoke: Vec<SensorData>,
        co: Vec<SensorData>,
        heat: Vec<SensorData>,
        #[serde(rename = "fire-button")]
        button: Vec<SensorData>,
        lpg: Vec<SensorData>,
    },
    #[serde(rename = "1")]
    Interrupt {
        token: Token,
        #[serde(rename = "fire")]
        fire_data: Vec<SensorData>,
        smoke: Vec<SensorData>,
        co: Vec<SensorData>,
        heat: Vec<SensorData>,
        #[serde(rename = "fire-button")]
        button: Vec<SensorData>,
        lpg: Vec<SensorData>,
    },
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub struct SensorData {
    pub id: u32,
    pub component: u32,
    pub value: u32,
    pub alert: FireStatus,
}

#[cfg(test)]
mod deserialize_tests {
    use super::{FireMQTTMessage, FireStatus, SensorData, Token};

    #[test]
    fn deserialize_periodic_data() {
        let input = r#"{
            "kind": "0",
            "payload": {
                "token": "abcd",
                "fire": [
                    {
                        "id": 0,
                        "component": 8,
                        "value": 460,
                        "alert": 0
                    }
                ],
                "smoke": [
                    {
                        "id": 0,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    },
                    {
                        "id": 0,
                        "component": 1,
                        "value": 240,
                        "alert": 0
                    },
                    {
                        "id": 1,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    },
                    {
                        "id": 2,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    },
                    {
                        "id": 0,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    }
                ],
                "co": [
                    {
                        "id": 0,
                        "component": 4,
                        "value": 460,
                        "alert": 0
                    },
                    {
                        "id": 1,
                        "component": 4,
                        "value": 460,
                        "alert": 0
                    }
                ],
                "heat": [
                    {
                        "id": 2,
                        "component": 2,
                        "value": 460,
                        "alert": 0
                    },
                    {
                        "id": 3,
                        "component": 2,
                        "value": 460,
                        "alert": 0
                    }
                ],
                "fire-button": [
                    {
                        "id": 1,
                        "component": 10,
                        "value": 1,
                        "alert": 0
                    }
                ],
                "lpg": [
                    {
                        "id": 1,
                        "component": 6,
                        "value": 20,
                        "alert": 0,
                    }
                ]
            }
        }"#;

        let result: FireMQTTMessage = serde_json::from_str(input).unwrap();

        let expected = FireMQTTMessage::Periodic {
            token: Token::from("abcd"),
            fire_data: vec![SensorData {
                id: 0,
                component: 8,
                value: 460,
                alert: FireStatus::SAFE,
            }],
            smoke: vec![
                SensorData {
                    id: 0,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: 1,
                    value: 240,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 2,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
            ],
            co: vec![
                SensorData {
                    id: 0,
                    component: 4,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: 4,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            heat: vec![
                SensorData {
                    id: 2,
                    component: 2,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 3,
                    component: 2,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            button: vec![SensorData {
                id: 1,
                component: 10,
                value: 1,
                alert: FireStatus::SAFE,
            }],
            lpg: vec![],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_interrupt_data() {
        let input = r#"{
            "kind": "1",
            "payload": {
                "token": "efgh",
                "fire": [
                    {
                        "id": 0,
                        "component": 8,
                        "value": 460,
                        "alert": 1
                    }
                ],
                "smoke": [],
                "co": [],
                "heat": [],
                "fire-button": [],
                "lpg": []
            }
        }"#;

        let result: FireMQTTMessage = serde_json::from_str(input).unwrap();

        let expected = FireMQTTMessage::Interrupt {
            token: Token::from("efgh"),
            fire_data: vec![SensorData {
                id: 0,
                component: 8,
                value: 460,
                alert: FireStatus::UNSAFE,
            }],
            smoke: vec![],
            co: vec![],
            heat: vec![],
            button: vec![],
            lpg: vec![]
        };

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod serialize_tests {

    use serde_json::json;

    use super::{FireMQTTMessage, FireStatus, SensorData, Token};

    #[test]
    fn serialize_periodic_data() {
        let input = FireMQTTMessage::Periodic {
            token: Token::from("abcd"),
            fire_data: vec![SensorData {
                id: 0,
                component: 8,
                value: 460,
                alert: FireStatus::SAFE,
            }],
            smoke: vec![
                SensorData {
                    id: 0,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: 1,
                    value: 240,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 2,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 0,
                    component: 0,
                    value: 120,
                    alert: FireStatus::SAFE,
                },
            ],
            co: vec![
                SensorData {
                    id: 0,
                    component: 4,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 1,
                    component: 4,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            heat: vec![
                SensorData {
                    id: 2,
                    component: 2,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
                SensorData {
                    id: 3,
                    component: 2,
                    value: 460,
                    alert: FireStatus::SAFE,
                },
            ],
            button: vec![SensorData {
                id: 1,
                component: 10,
                value: 1,
                alert: FireStatus::SAFE,
            }],
            lpg: vec![],
        };

        let expected = json!({
            "kind": "0",
            "payload": {
                "token": "abcd",
                "fire": [
                    {
                        "id": 0,
                        "component": 8,
                        "value": 460,
                        "alert": 0
                    }
                ],
                "smoke": [
                    {
                        "id": 0,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    },
                    {
                        "id": 0,
                        "component": 1,
                        "value": 240,
                        "alert": 0
                    },
                    {
                        "id": 1,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    },
                    {
                        "id": 2,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    },
                    {
                        "id": 0,
                        "component": 0,
                        "value": 120,
                        "alert": 0
                    }
                ],
                "co": [
                    {
                        "id": 0,
                        "component": 4,
                        "value": 460,
                        "alert": 0
                    },
                    {
                        "id": 1,
                        "component": 4,
                        "value": 460,
                        "alert": 0
                    }
                ],
                "heat": [
                    {
                        "id": 2,
                        "component": 2,
                        "value": 460,
                        "alert": 0
                    },
                    {
                        "id": 3,
                        "component": 2,
                        "value": 460,
                        "alert": 0
                    }
                ],
                "fire-button": [
                    {
                        "id": 1,
                        "component": 10,
                        "value": 1,
                        "alert": 0
                    }
                ],
                "lpg": [
                    {
                        "id": 1,
                        "component": 6,
                        "value": 20,
                        "alert": 0,
                    }
                ],
            }
        });

        let result = serde_json::to_string_pretty(&input).unwrap();

        assert_eq!(result, serde_json::to_string_pretty(&expected).unwrap());
    }

    #[test]
    fn serialize_interrupt_data() {
        let input = FireMQTTMessage::Interrupt {
            token: Token::from("efgh"),
            fire_data: vec![SensorData {
                id: 0,
                component: 8,
                value: 460,
                alert: FireStatus::UNSAFE,
            }],
            smoke: vec![],
            co: vec![],
            heat: vec![],
            button: vec![],
            lpg: vec![],
        };

        let expected = json!({
            "kind": "1",
            "payload": {
                "token": "efgh",
                "fire": [
                    {
                        "id": 0,
                        "component": 8,
                        "value": 460,
                        "alert": 1
                    }
                ],
                "smoke": [],
                "co": [],
                "heat": [],
                "fire-button": [],
                "lpg": [],
            }
        });

        let result = serde_json::to_string_pretty(&input).unwrap();

        assert_eq!(result, serde_json::to_string_pretty(&expected).unwrap());
    }
}
