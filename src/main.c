#include "main.h"

int main(int argc, char* argv[]) {

  SimpleInode * inode = parseDir("..");
  printInode(inode, 0);
  /* char c; */
  /* terminalSetup(); */
  /* while (1) { */
  /*   char c = '\0'; */
  /*   read(STDIN_FILENO, &c, 1); */
  /*   if (iscntrl(c)) { */
  /*     printf("%d\r\n", c); */
  /*   } else { */
  /*     printf("%d ('%c')\r\n", c, c); */
  /*   } */
  /*   if (c == 'q') break; */
  /* } */
  return 0;
}
