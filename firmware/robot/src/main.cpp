#include <Arduino.h>

#include "util.h"
#include "protocol.h"
#include "net.h"

// 34:85:18:A9:CF:E4
uint8_t controller_address[] = {0x34, 0x85, 0x18, 0xA9, 0xCF, 0xE4};

void on_data_recv(const uint8_t *mac, const uint8_t *incoming_data, int len) {
    control_message message;
    memcpy(&message, incoming_data, sizeof(message));

    Serial.print("Received message from controller: ");
    Serial.print(message.x1);
    Serial.print(", ");
    Serial.print(message.y1);
    Serial.print(", ");
    Serial.print(message.x2);
    Serial.print(", ");
    Serial.println(message.y1);
}

void setup() {
    Serial.begin(9600);

    OK_OR_RETURN(net_init());
    OK_OR_RETURN(net_add_peer(controller_address));
    net_register_callbacks();
}

void loop() {
    status_message message;
    message.robot_id = 1;
    message.flags = 0;

    Serial.println("Sending message to controller");

    net_send_message_to_all(&message, sizeof(message));
    delay(1000);
}
