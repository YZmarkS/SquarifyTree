#include "main.h"

int main(int argc, char* argv[]) {

  SimpleInode * inode = parseDir(".");
  printInode(inode, 0);
  printf("=====================\n");
  Rectangle* rect = fill(inode->detail.dir_detail.children, 200, 100);
  debugRectangle(rect, 0);

  uint32_t* matrix = (uint32_t*)calloc(200 * 51, sizeof(uint32_t));

  clearMatrix(200, 51, matrix);
  drawBoarder(200, 51, matrix);
  printMatrix(200, 51, matrix);
  renderRectangles(rect, 0, 0, 200, 51, matrix);
  printMatrix(200, 51, matrix);


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
