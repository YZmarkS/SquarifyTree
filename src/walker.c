#include "walker.h"

SimpleInode * parseReg(const char * path /* borrow */) {
  struct stat statbuf;
  int stat_result = stat(path, &statbuf);
  if (stat_result != 0) { printf("Cannot access inode\n"); exit(1); }

  SimpleInode * reg_inode = (SimpleInode *) malloc(sizeof(SimpleInode));
  if (reg_inode == NULL) { printf("Cannot allocate memory for file\n"); exit(1); }

  reg_inode->type = REG_TYPE;
  reg_inode->path = strdup(path);
  reg_inode->size = (off64_t) statbuf.st_size;
  reg_inode->detail.reg_detail.size = statbuf.st_size;

  return reg_inode;
}

SimpleInode * parseDir(const char * path /* borrow */) {
  SimpleInode * dir_inode = (SimpleInode *)malloc(sizeof(SimpleInode));
  if (reg_inode == NULL) { printf("Cannot allocate memory for directory\n"); exit(1); }
  SimpleInode * childrens = NULL;

  DIR * dir_stream = opendir(path);
  if (dir_stream == NULL) { printf("Cannot open directory\n"); exit(1); }

  struct dirent* dir_entry = readdir(dir_stream)
  while (dir_entry != NULL) {

  }

}
