#ifndef SUMO_NET_H
#define SUMO_NET_H

void on_data_recv(const uint8_t *mac, const uint8_t *incoming_data, int len);

int net_init();
void net_register_callbacks();
int net_add_peer(uint8_t *peer_address);
int net_send_message_to_all(void *message, size_t message_size);
int net_send_message_to_peer(uint8_t *peer_address, void *message, size_t message_size);

#endif