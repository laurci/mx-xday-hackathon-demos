#include <Arduino.h>
#include <WiFi.h>

void setup(){
  Serial.begin(9600);
}
 
void loop(){
  Serial.print("MAC Address:  ");
  Serial.println(WiFi.macAddress());
  delay(1000);
}