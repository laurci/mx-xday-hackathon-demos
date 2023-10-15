#include <Arduino.h>
#include <WiFi.h>
#include <esp_now.h>

#include "protocol.h"
#include "net.h"

esp_now_peer_info_t peer_info;

int net_init() {
    WiFi.mode(WIFI_STA);
    
    if (esp_now_init() != ESP_OK) {
        Serial.println("Error initializing ESP-NOW");
        return -1;
    }

    return 0;
}

int net_add_peer(uint8_t *peer_address) {
    memcpy(peer_info.peer_addr, peer_address, 6);
    peer_info.channel = 0;  
    peer_info.encrypt = false;

    if (esp_now_add_peer(&peer_info) != ESP_OK){
        Serial.println("Failed to add peer");
        return -1;
    }

    return 0;
}

int net_send_message_to_all(void *message, size_t message_size) {
    esp_err_t result = esp_now_send(0, (uint8_t *) message, message_size);
    if(result != ESP_OK) {
        Serial.println("Error sending message");
        return -1;
    }

    return 0;
}

int net_send_message_to_peer(uint8_t *peer_address, void *message, size_t message_size) {
    esp_err_t result = esp_now_send(peer_address, (uint8_t *) message, message_size);
    if(result != ESP_OK) {
        Serial.println("Error sending message");
        return -1;
    }

    return 0;
}

void net_register_callbacks() {
    esp_now_register_recv_cb(on_data_recv);
}