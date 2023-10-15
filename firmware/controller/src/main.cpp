#include <Arduino.h>

#include "util.h"
#include "protocol.h"
#include "net.h"

// EC:DA:3B:62:48:0C
uint8_t robot_1_address[] = {0xEC, 0xDA, 0x3B, 0x62, 0x48, 0x0C};

// uint8_t robot_2_address[] = {0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF};

void on_data_recv(const uint8_t *mac, const uint8_t *incoming_data, int len) {
    status_message message;
    memcpy(&message, incoming_data, sizeof(message));

    Serial.print("Received message from ");
    Serial.print(message.robot_id);
    Serial.print(" with status ");
    Serial.println(message.flags);
}

void setup() {
    Serial.begin(9600);

    OK_OR_RETURN(net_init());
    OK_OR_RETURN(net_add_peer(robot_1_address));
    // OK_OR_RETURN(net_add_peer(robot_2_address));
    net_register_callbacks();
}

void loop() {
    control_message message;
    message.x1 = 1.0f;
    message.y1 = 2.0f;
    message.x2 = 3.0f;
    message.y2 = 4.0f;

    net_send_message_to_all(&message, sizeof(message));
    delay(1000);
}
