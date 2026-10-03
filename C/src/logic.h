#ifndef _LOGIC
#define _LOGIC

#include <stdlib.h>
#include <stdio.h>

#include "simple_inode.h"
#include "squarify.h"
#include "layout_logic.h"
#include "user_input.h"
#include "util.h"

typedef struct AppState {
  int temp;
};

typedef struct ViewModeState {
  SimpleInode* starting_inode;
  InodePtrList* history_stack;

  Layout* layout;
  Cell* active_cell;

  SimpleInode* all_inodes;
} ViewModeState;

typedef struct FindModeState {
} FindModeState;

#endif
