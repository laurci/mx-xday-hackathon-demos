
#include <esp_now.h>
#include <WiFi.h>
#include <ESP32Servo.h>
#include <esp_wifi.h>


// select target 1 for blue, 2 for orange
// they differ a bit, God didn't make all servos and sensors equal

#define ROBOT_ID 1
// #define ROBOT_ID 2

uint8_t MACAddress0[] = {0x00, 0x01, 0x02, 0x03, 0x03, 0x00};

uint8_t MACAddress1[] = {0x00, 0x01, 0x02, 0x03, 0x04, 0x00};
uint8_t MACAddress2[] = {0x00, 0x01, 0x02, 0x03, 0x04, 0x01};

int robo_lost_th[] = {0, 4050};

esp_now_peer_info_t base;

typedef struct
{
	int j1x, j1y, j2x, j2y;
} JData;

JData data;
Servo s1, s2;

void OnDataRecv(const uint8_t *mac, const uint8_t *incomingData, int len)
{
	memcpy(&data, incomingData, sizeof(data));

	// int throttle = map(data.j1y, 0, 1000, -100, 100);
	int throttle = data.j1y;
	int dir = data.j2x;

	if (ROBOT_ID == 1)
	{
		s1.writeMicroseconds(1475 - throttle + dir);
		s2.writeMicroseconds(1455 + throttle + dir);
	}

	if (ROBOT_ID == 2)
	{
		s1.writeMicroseconds(1485 - throttle + dir);
		s2.writeMicroseconds(1490 + throttle + dir);
	}
}

void setup()
{

	// pinMode(2, OUTPUT);
	// digitalWrite(2, HIGH);

	s1.attach(26);
	s2.attach(27);

	// Initialize Serial Monitor
	Serial.begin(115200);

	// Set device as a Wi-Fi Station
	WiFi.mode(WIFI_STA);
	if (ROBOT_ID == 1)
	{
		esp_wifi_set_mac(WIFI_IF_STA, &MACAddress1[0]);
	}

	if (ROBOT_ID == 2)
	{
		esp_wifi_set_mac(WIFI_IF_STA, &MACAddress2[0]);
	}

	// Init ESP-NOW
	if (esp_now_init() != ESP_OK)
	{
		Serial.println("Error initializing ESP-NOW");
		return;
	}

	esp_now_register_recv_cb(OnDataRecv);

	memcpy(base.peer_addr, MACAddress0, 6);
	base.channel = 0;
	base.encrypt = false;

	// Add peer
	if (esp_now_add_peer(&base) != ESP_OK)
	{
		Serial.println("Failed to add base");
		return;
	}
}

float sensor;

void loop()
{

	sensor = analogRead(34) * 0.05 + sensor * 0.95;
	// Serial.println(sensor);

	if (sensor > robo_lost_th[1])
	{
		uint8_t id = ROBOT_ID;
		if (esp_now_send(MACAddress0, &id, sizeof(id)))
		{
			Serial.println("Failed to send to base");
		}
		Serial.println("I lost!");
		delay(500);

		int throttle = 0;
		int dir = 0;

		if (ROBOT_ID == 1)
		{
			s1.writeMicroseconds(1475 - throttle + dir);
			s2.writeMicroseconds(1455 + throttle + dir);
		}

		if (ROBOT_ID == 2)
		{
			s1.writeMicroseconds(1485 - throttle + dir);
			s2.writeMicroseconds(1490 + throttle + dir);
		}

		delay(500);
	}

	delay(15);
}