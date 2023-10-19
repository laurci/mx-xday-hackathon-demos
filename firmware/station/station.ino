#include "Arduino.h"
#include "TFT_eSPI.h"
#include "pin_config.h"
#include <esp_now.h>
#include <esp_wifi.h>
#include <WiFi.h>

uint8_t MACAddress0[] = {0x00, 0x01, 0x02, 0x03, 0x03, 0x00};

uint8_t MACAddress1[] = {0x00, 0x01, 0x02, 0x03, 0x04, 0x00};
uint8_t MACAddress2[] = {0x00, 0x01, 0x02, 0x03, 0x04, 0x01};

typedef struct
{
	int j1x, j1y, j2x, j2y;
} JData;

JData data;

TFT_eSPI tft = TFT_eSPI();
TFT_eSprite sprite = TFT_eSprite(&tft);

esp_now_peer_info_t robo1, robo2;

uint8_t loser;

void OnDataRecv(const uint8_t *mac, const uint8_t *incomingData, int len)
{
	memcpy(&loser, incomingData, sizeof(loser));

	Serial.println("lost_" + String(loser));
}

void setup()
{
	Serial.begin(115200);
	Serial.setTimeout(10);
	pinMode(PIN_POWER_ON, OUTPUT);
	digitalWrite(PIN_POWER_ON, HIGH);
	pinMode(PIN_TOUCH_RES, OUTPUT);
	digitalWrite(PIN_TOUCH_RES, LOW);
	delay(500);
	tft.begin();
	sprite.createSprite(170, 320);
	sprite.setTextColor(TFT_WHITE, TFT_BLACK);
	sprite.fillRect(0, 0, 170, 320, TFT_BLACK);
	sprite.pushSprite(0, 0);

	WiFi.mode(WIFI_STA);
	esp_wifi_set_mac(WIFI_IF_STA, &MACAddress0[0]);

	if (esp_now_init() != ESP_OK)
	{
		Serial.println("Error initializing ESP-NOW");
		return;
	}

	esp_now_register_recv_cb(OnDataRecv);

	memcpy(robo1.peer_addr, MACAddress1, 6);
	robo1.channel = 0;
	robo1.encrypt = false;

	memcpy(robo2.peer_addr, MACAddress2, 6);
	robo2.channel = 0;
	robo2.encrypt = false;

	// Add peer
	if (esp_now_add_peer(&robo1) != ESP_OK)
	{
		Serial.println("Failed to add robo1");
		return;
	}

	if (esp_now_add_peer(&robo2) != ESP_OK)
	{
		Serial.println("Failed to add robo2");
		return;
	}

	delay(1000);
}

void drawProgressBar(int value, int min, int max, int width, int height, bool vertical, int x, int y, TFT_eSprite &sprite, uint32_t color)
{

	sprite.fillRect(x, y, width, height, TFT_BLACK);

	sprite.drawRect(x, y, width, height, color);

	int fillx, filly, fillh, fillw;
	if (vertical)
	{
		filly = map(value, max, min, y + 2, y + height - 2);
		fillh = map(value, min, max, 0, height - 4);

		sprite.fillRect(x + 2, filly, width - 4, fillh, TFT_ORANGE);
	}
	else
	{
		fillx = map(value, min, max, x + 2, x + width - 2);
		fillw = map(value, min, max, 0, width - 4);

		sprite.fillRect(x + 2, y + 2, fillw, height - 4, TFT_ORANGE);
	}
}

void drawJoystick(int valx, int xmin, int xmax, int valy, int ymin, int ymax, int x, int y, int size, TFT_eSprite &sprite, uint32_t color)
{
	int tlx = x - size / 2;
	int tly = y - size / 2;
	int relative_x = map(valx, xmin, xmax, 0, size);
	int relative_y = map(valy, ymin, ymax, size, 0);

	sprite.fillRect(tlx - 4, tly - 4, size + 9, size + 9, TFT_BLACK);
	sprite.drawCircle(x, y, size / 2, color);
	sprite.drawCircle(tlx + relative_x, tly + relative_y, 3, color);
	sprite.drawLine(tlx + relative_x, tly - 4, tlx + relative_x, y + size / 2 + 4, color); // vertical
	sprite.drawLine(tlx - 4, tly + relative_y, x + size / 2 + 4, tly + relative_y, color); // horizontal

	// drawProgressBar(valx, xmin, xmax, size + 8, 8, false, tlx - 4, tly + size + 5, sprite, color);
	// drawProgressBar(valy, ymin, ymax, 8, size + 8, true, tlx + size + 5, tly - 4, sprite, color);
}

void loop()
{
	char str[50];
	int8_t num[4];
	int x1 = 0, y1 = 0, x2 = 0, y2 = 0;
	esp_err_t result;

	memset(str, 0, 50);
	if (Serial.available())
	{
		Serial.readBytesUntil('\n', str, 50);

		if (str[0] == 'p')
		{

			sscanf(str, "p %d %d %d %d", &y1, &x1, &y2, &x2);

			Serial.println(String(y1) + String(x1) + String(y2) + String(x2));

			data.j1y = y1;
			data.j2x = x1;
			result = esp_now_send(MACAddress1, (uint8_t *)&data, sizeof(data));

			if (result == ESP_OK)
			{
				sprintf(str, "SEND");
				sprite.textbgcolor = TFT_ORANGE;
				sprite.textcolor = TFT_BLACK;
				sprite.drawString(str, 0, 1, 1);
				sprite.textbgcolor = TFT_BLACK;
				sprite.textcolor = TFT_ORANGE;
			}
			else
			{
				sprintf(str, "ERR!");
				sprite.textbgcolor = TFT_ORANGE;
				sprite.textcolor = TFT_BLACK;
				sprite.drawString(str, 0, 1, 1);
				sprite.textbgcolor = TFT_BLACK;
				sprite.textcolor = TFT_ORANGE;
			}

			data.j1y = y2;
			data.j2x = x2;
			result = esp_now_send(MACAddress2, (uint8_t *)&data, sizeof(data));

			if (result == ESP_OK)
			{
				sprintf(str, "SEND");
				sprite.textbgcolor = TFT_ORANGE;
				sprite.textcolor = TFT_BLACK;
				sprite.drawString(str, 30, 0, 1);
				sprite.textbgcolor = TFT_BLACK;
				sprite.textcolor = TFT_ORANGE;
			}
			else
			{
				sprintf(str, "ERR!");
				sprite.textbgcolor = TFT_ORANGE;
				sprite.textcolor = TFT_BLACK;
				sprite.drawString(str, 30, 0, 1);
				sprite.textbgcolor = TFT_BLACK;
				sprite.textcolor = TFT_ORANGE;
			}
		}

		if (Serial.availableForWrite())
		{
		}

		sprite.textcolor = TFT_WHITE;

		drawJoystick(x1, -100, 100, y1, -100, 100, 40, 50, 60, sprite, TFT_ORANGE);
		sprintf(str, "x:%5d", x1);
		sprite.drawString(str, 6, 90, 1);

		sprintf(str, "y:%5d", y1);
		sprite.drawString(str, 6, 100, 1);

		drawJoystick(x2, -100, 100, y2, -100, 100, 130, 50, 60, sprite, TFT_ORANGE);
		sprintf(str, "x:%5d", x2);
		sprite.drawString(str, 120, 90, 1);

		sprintf(str, "y:%5d", y2);
		sprite.drawString(str, 120, 100, 1);

		sprite.pushSprite(0, 0);
	}
}