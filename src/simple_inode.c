#include "simple_inode.h"

void printInodesArray(SimpleInode* inodes[], long length) {
  for (long i = 0; i < length; i++) {
    printf("%ld ", inodes[i]->size);
  }
  printf("\n");
}

void printInode(SimpleInode* inode, int depth) {
  for (int i = 0; i < depth; i++) {
    printf("  ");
  }
  printf("%s, %ld\n", basename(inode->path), inode->size);
  if (inode->type == DIR_TYPE) {
    SimpleInode * children = inode->detail.dir_detail.children;
    while (children != NULL) {
      printInode(children, depth + 1);
      children = children->next;
    }
  }
}

InodePtrSlice listToSlice(SimpleInode* list) {
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

  InodePtrSlice result = { array, length };
  return result;
}

SimpleInode* sliceToList(InodePtrSlice slice) {
  long length = slice.length;
  SimpleInode* head = NULL;
  for (long i = length - 1; i >= 0; i--) {
    slice.content[i]->next = head;
    head = slice.content[i];
  }
  return head;
}

void quicksort(SimpleInode* inodes[], long lo, long hi, long total) {
  printf("===\n");
  printInodesArray(inodes, total);
  if (hi <= lo) {
    return;
  }
  SimpleInode* pivot_inode = inodes[hi];
  off_t pivot_size = pivot_inode->size;

  long i = lo - 1, j = hi;
  SimpleInode* i_inode;
  SimpleInode* j_inode;

  while (true) {
    while (inodes[--j]->size < pivot_size) {
      if (j == lo) { break; }
    }
    while (inodes[++i]->size > pivot_size) {
      if (i == hi) { break; }
    }

    if (j <= i) { break; }
    SimpleInode* i_inode = inodes[i];
    SimpleInode* j_inode = inodes[j];
    inodes[j] = i_inode;
    inodes[i] = j_inode;
  }
  i_inode = inodes[i];
  inodes[hi] = i_inode;
  inodes[i] = pivot_inode;

  printf("i: %ld, hi: %ld\n", i, hi);
  printInodesArray(inodes, total);

  quicksort(inodes, lo, i - 1, total);
  quicksort(inodes, i + 1, hi, total);
}

SimpleInode* sortList(SimpleInode* list) {
  SimpleInode* result = NULL;
  InodePtrSlice slice = listToSlice(list);
  quicksort(slice.content, 0, slice.length - 1, slice.length);
  result = sliceToList(slice);
  free(slice.content);
  return result;
}

long inodeListLength(SimpleInode* list) {
  long length = 0;
  while (list != NULL) {
    length++;
    list = list->next;
  }
  return length;
}

off_t sumInodeListSize(SimpleInode* list) {
  off_t total = 0;
  while (list != NULL) {
    total += list->size;
    list = list->next;
  }
  return total;
}

off_t sumFirstNInodesSize(SimpleInode* list, long n) {
  off_t total = 0;
  for (int i = 0; i < n; i++) {
    if (list == NULL) { printf("Ran out of list\n"); exit(1); }
    total += list->size;
    list = list->next;
  }
  return total;
}
