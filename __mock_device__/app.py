import time
import paho.mqtt.client as mqtt
import json

id = "84ac5dd0-94b4-424c-ae5a-96a4cacdc093"

client_metrics_topic = id + "/fire-alert-metrics"
server_command_topic = id + "/<feature>-commands"
client_name = id

mqtt_client = mqtt.Client(client_name)
mqtt_client.connect("test.mosquitto.org")

mqtt_client.loop_start()

for i in range(5):
    message = {
        "kind": "0",
        "payload": {
            "token": "eyJhbGciOiJIUzI1NiJ9.eyJjbGllbnRfaWQiOiI4NGFjNWRkMC05NGI0LTQyNGMtYWU1YS05NmE0Y2FjZGMwOTMiLCJub25jZSI6ImU1ZjY2YzJiLWMxMzItNGY0NS1hMjU4LTUyMzhhNzAwOWFhZCJ9.u25Vv9Ct8FrIbSHSo2-59n-7wLhpyvr_WtVrfNhObZc",
            "fire": [{"id": 0, "component": 8, "value": 460, "alert": 0}],
            "smoke": [
                {"id": 0, "component": 0, "value": 120, "alert": 0},
                {"id": 0, "component": 1, "value": 240, "alert": 0},
                {"id": 1, "component": 0, "value": 120, "alert": 0},
                {"id": 2, "component": 0, "value": 120, "alert": 0},
                {"id": 0, "component": 0, "value": 120, "alert": 0},
            ],
            "co": [
                {"id": 0, "component": 4, "value": 460, "alert": 0},
                {"id": 1, "component": 4, "value": 460, "alert": 0},
            ],
            "heat": [
                {"id": 2, "component": 2, "value": 460, "alert": 0},
                {"id": 3, "component": 2, "value": 460, "alert": 0},
            ],
            "fire-button": [{"id": 1, "component": 10, "value": 1, "alert": 0}],
            "lpg": [
                {
                    "id": 1,
                    "component": 6,
                    "value": 20,
                    "alert": 0,
                }
            ],
        },
    }
    mqtt_client.publish(client_metrics_topic, json.dumps(message))
    print(i)

    time.sleep(1)
