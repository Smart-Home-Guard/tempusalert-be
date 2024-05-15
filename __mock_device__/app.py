import time
import paho.mqtt.client as mqtt
import json

id = "248781fd-5ebb-4395-b90e-f0721039ebfe"

client_metrics_topic = id + "/device-status-metrics"
server_command_topic = id + "/<feature>-commands"
client_name = id

mqtt_client = mqtt.Client(client_name)
mqtt_client.connect("14.225.192.183")

mqtt_client.loop_start()

for i in range(5):
    message = {
        "kind": "2",
        "payload": {
            "token": "eyJhbGciOiJIUzI1NiJ9.eyJjbGllbnRfaWQiOiIyNDg3ODFmZC01ZWJiLTQzOTUtYjkwZS1mMDcyMTAzOWViZmUiLCJub25jZSI6IjAxOGY3YTUwLTA1N2QtNzk1YS05ZDg1LTcwZTg2YWUyYmQ2OSJ9.UgMQHccNA9NUuKUYEiPwf6RelXO-zlXM1SK5aDivJyA",
            "data": [
                {"id": i + 1, "component": i * 100 + 1, "kind": 50},
                {"id": i, "component": i * 100 + 2, "kind": 53},
                {"id": i, "component": i * 100 + 5, "kind": 52},
                {"id": i, "component": i * 100 + 6, "kind": 51},
                {"id": i + 1, "component": i * 100 + 4, "kind": 54},
                {"id": i + 1, "component": i * 100 + 12, "kind": 54},
                {"id": i + 1, "component": i * 100 + 17, "kind": 55},
                {"id": i + 2, "component": i * 100 + 13, "kind": 56},
                {"id": i + 2, "component": i * 100 + 24, "kind": 57},
                {"id": i + 3, "component": i * 100 + 23, "kind": 55},
                {"id": i + 4, "component": i * 100 + 25, "kind": 50},
                {"id": i + 3, "component": i * 100 + 26, "kind": 52},
                {"id": i + 4, "component": i * 100 + 27, "kind": 51},
                {"id": i + 10, "component": i * 100 + 27, "kind": 52},
                {"id": i + 12, "component": i * 100 + 27, "kind": 53},
            ],
        },
    }
    mqtt_client.publish(client_metrics_topic, json.dumps(message))
    print(i)

    time.sleep(1)
