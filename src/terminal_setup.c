#include "terminal_setup.h"

struct termios old_termios;

void disableRawMode() {
  tcsetattr(STDIN_FILENO, TCSAFLUSH, &old_termios);
}

void normalTearDown() {
  disableRawMode();
  printf("\x1b[?1049l");
}

void enableRawMode() {
  tcgetattr(STDIN_FILENO, &old_termios);

  struct termios new_termios = old_termios;

  tcflag_t output_flags_to_disable = 0x0
    | OPOST;
  tcflag_t input_flags_to_disable = 0x0
    | BRKINT
    | ICRNL
    | INPCK
    | ISTRIP
    | IXON;
  tcflag_t control_flags_to_enable = 0x0
    | CS8;
  tcflag_t local_flags_to_disable = 0x0
    | ECHO
    | ICANON
    | ISIG;

  new_termios.c_iflag &= ~input_flags_to_disable;
  new_termios.c_oflag &= ~output_flags_to_disable;
  new_termios.c_cflag |= ~control_flags_to_enable;
  new_termios.c_lflag &= ~local_flags_to_disable;

  // This actually overwrites the meaning of [read] returning 0
  // read(2) says if [read] return 0 it is end of file
  // terminos(3) says if [read] return 0 it is because stream is empty
  new_termios.c_cc[VMIN] = 0;
  new_termios.c_cc[VTIME] = 1;


  tcsetattr(STDIN_FILENO, TCSAFLUSH, &new_termios);
}


void terminalSetup() {
  enableRawMode();

  // Alternate screen
  printf("\x1b[?1049h");
  fflush(stdout);

  atexit(normalTearDown);
}
