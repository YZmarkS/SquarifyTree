#include "squarify.h"

void quicksort(SimpleInode* inodes[], long lo, long hi) {
  if (lo >= hi) {
    return;
  }
  SimpleInode* pivot_inode = inodes[lo];
  off_t pivot_size = pivot_inode->size;

  long i = lo, j = hi + 1;
  SimpleInode* i_inode;
  SimpleInode* j_inode;

  while (true) {
    while (inodes[++i]->size < pivot_size) {
      if (i == hi) { break; }
    }
    while (inodes[--j]->size > pivot_size) {
      if (j == lo) { break; }
    }
    if (i >= j) { break; }
    SimpleInode* i_inode = inodes[i];
    SimpleInode* j_inode = inodes[j];
    inodes[j] = i_inode;
    inodes[i] = j_inode;
  }
  j_inode = inodes[j];
  inodes[lo] = j_inode;
  inodes[j] = pivot_inode;

  quicksort(inodes, lo, j - 1);
  quicksort(inodes, j + 1, hi);
}

ArrayifyResult listToArray(SimpleInode* list) {
  long length = 0;
  SimpleInode* pointer = list;
  while (pointer != NULL) {
    length++;
    pointer = pointer->next;
  }
  SimpleInode** array = (SimpleInode**)calloc(length, sizeof(SimpleInode*));
  pointer = list;
  for (long i = 0; i < length; i++) {
    array[i] = pointer;
    pointer = pointer->next;
  }

  ArrayifyResult result = { array, length };
  return result;
}

// The function expects a Rectangle with a set dimension and orientation
// It also assume the next field is NULL
void fill(SimpleInode * remaining, Rectangle* rectangle) {

  ArrayifyResult array = listToArray(remaining);
  quicksort(array.content, 0, array.length - 1);


}
