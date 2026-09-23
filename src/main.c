#include "main.h"

int main(int argc, char* argv[]) {

  SimpleInode * inode = parseDir(".");
  printInode(inode, 0);
  printf("=====================\n");
  Rectangle* rect = fill(inode->detail.dir_detail.children, 200, 120, LANDSCAPE);
  validateRectangle(rect, 0);
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
  printf("\u2597\u2500\u2596\n");
  printf("\u2502 \u2502\n");

  return 0;
}
