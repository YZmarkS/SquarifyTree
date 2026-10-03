#include "user_input.h"

/* This function only handles a subset of all key press
   The proper implementation is better left to professional
 */
KeyPressed readOneKeyPress() {
  KeyPressed key_pressed;
  key_pressed.type = NOTHING;

  uint8_t c;
  ssize_t bytes_read;
  bytes_read = read(STDIN_FILENO, &c, 1);

  if (bytes_read == 1) {
    bool is_control_char = iscntrl(c);

    if (!is_control_char) {
      key_pressed.type = ASCII;
      key_pressed.key_content.ascii_char = c;
    }
    if (c == '\xd') {
      key_pressed.type = ENTER;
    }
    if (c == '\xff') {
      key_pressed.type = BACKSPACE;
    }
    if (c == '\x1b') {

      uint8_t seq[3];
      if (read(STDIN_FILENO, &seq[0], 1) != 1) {
	return key_pressed;
      }
      if (read(STDIN_FILENO, &seq[1], 1) != 1) {
	return key_pressed;
      }

      if (seq[0] == '[') {
	switch (seq[1]) {
	case 'A':
	  key_pressed.type = ARROW;
	  key_pressed.key_content.arrow_direction = U;
	  break;
	case 'B':
	  key_pressed.type = ARROW;
	  key_pressed.key_content.arrow_direction = D;
	  break;
	case 'C':
	  key_pressed.type = ARROW;
	  key_pressed.key_content.arrow_direction = R;
	  break;
	case 'D':
	  key_pressed.type = ARROW;
	  key_pressed.key_content.arrow_direction = L;
	  break;
	}
      }
    }
  }

  return key_pressed;
}
