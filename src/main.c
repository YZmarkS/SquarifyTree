#include "main.h"

int main(int argc, char* argv[]) {

  SimpleInode * inode = parseDir(".");
  printInode(inode, 0);
  printf("=====================\n");
  Rectangle* rect = fill(inode->detail.dir_detail.children, 200, 50);
  debugRectangle(rect, 0);

  uint32_t* matrix = (uint32_t*)calloc(201 * 51, sizeof(uint32_t));

  clearMatrix(201, 51, matrix);
  drawBoarder(201, 51, matrix);
  renderRectangles(rect, 0, 0, 201, 51, matrix);
  printMatrix(201, 51, matrix);

  CellPtrList* cell_ptr_list = rectangleToCells(rect);
  printf("DONE conversion\n");

  relateCells(cell_ptr_list, 200, LEFT_RIGHT);
  relateCells(cell_ptr_list, 50, TOP_BOTTOM);

  while (cell_ptr_list != NULL) {
    clearMatrix(201, 51, matrix);
    drawBoarder(201, 51, matrix);
    renderRectangles(rect, 0, 0, 201, 51, matrix);
    Cell* cell = cell_ptr_list->cell_ptr;
    int x = cell->top_left.x, y = cell->top_left.y;
    matrix[(y + 1) * 201 + (x + 1)] = '*';
    Cell* top = cell->neighbours.top;
    Cell* bottom = cell->neighbours.bottom;
    Cell* left = cell->neighbours.left;
    Cell* right = cell->neighbours.right;

    if (top != NULL) {
      matrix[(top->top_left.y + 1) * 201 + (top->top_left.x + 1)] = 'T';
    }
    if (bottom != NULL) {
      matrix[(bottom->top_left.y + 1) * 201 + (bottom->top_left.x + 1)] = 'B';
    }
    if (left != NULL) {
      matrix[(left->top_left.y + 1) * 201 + (left->top_left.x + 1)] = 'L';
    }
    if (right != NULL) {
      matrix[(right->top_left.y + 1) * 201 + (right->top_left.x + 1)] = 'R';
    }
    printMatrix(201, 51, matrix);

    cell_ptr_list = cell_ptr_list->next;
  }




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
