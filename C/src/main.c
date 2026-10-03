#include "main.h"

int main(int argc, char* argv[]) {

  int mx = 200;
  int my = 50;

  uint32_t* matrix = (uint32_t*)calloc((mx + 1) * (my + 1), sizeof(uint32_t));

  SimpleInode* inode = parseDir(".");
  SimpleInode* children = inode->detail.dir_detail.children;

  SimpleInode* curr_biggest_inode = children;
  InodePtrList* history_stack = NULL;
  Layout* layout = squarify(inode->detail.dir_detail.children, mx, my);
  CellPtrList* cell_ptr_list = layoutToCells(layout);
  Cell* curr_cell = cell_ptr_list->cell_ptr;



  relateCells(cell_ptr_list, mx, LEFT_RIGHT);
  relateCells(cell_ptr_list, my, TOP_BOTTOM);

  terminalSetup();
  while (1) {

    KeyPressed key_pressed = readOneKeyPress();

    switch(key_pressed.type) {
    case ARROW:
      break;
    case ENTER:
      break;
    case BACKSPACE:
      break;
    case ASCII:
      break;
    default:
      break;
    }


  }
  return 0;
}
