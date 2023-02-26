#include <stdlib.h>

int string_to_int(const char *num) {
  return atoi(num);
}

static int get_first_char(const char* str) {
  return str[0];
}

int test_note() {
  return get_first_char(nullptr);
}

void ls() {
  system("ls");
}
