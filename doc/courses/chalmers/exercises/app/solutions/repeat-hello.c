#include <stdbool.h>
#include <stdio.h>

#include <timer.h>

int main (void) {
  while (true) {
    led_toggle(0);
    delay_ms(500);
  }
}

