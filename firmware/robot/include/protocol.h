#ifndef SUMO_PROTOCOL_H
#define SUMO_PROTOCOL_H

struct control_message {
  float x1;
  float y1;
  float x2;
  float y2;
};

struct status_message {
  int robot_id; // 1 / 2
  int flags;
};


#endif