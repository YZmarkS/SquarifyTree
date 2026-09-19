#include "walker.h"

void printInode(SimpleInode * inode, int depth) {
  for (int i = 0; i < depth; i++) {
    printf("  ");
  }
  printf("%s\n", basename(inode->path));
  if (inode->type == DIR_TYPE) {
    SimpleInode * children = inode->detail.dir_detail.children;
    while (children != NULL) {
      printInode(children, depth + 1);
      children = children->next;
    }
  }
}

SimpleInode * parseReg(const char * path /* borrow */) {
  struct stat statbuf;
  int stat_result = stat(path, &statbuf);
  if (stat_result != 0) { printf("Cannot access inode\n"); exit(1); }

  SimpleInode * reg_inode = (SimpleInode *)malloc(sizeof(SimpleInode));
  if (reg_inode == NULL) { printf("Cannot allocate memory for file\n"); exit(1); }

  reg_inode->type = REG_TYPE;
  reg_inode->path = strdup(path);
  reg_inode->size = statbuf.st_size;
  reg_inode->detail.reg_detail.size = statbuf.st_size;

  return reg_inode;
}

SimpleInode * parseDir(const char * path /* borrow */) {
  SimpleInode * dir_inode = (SimpleInode *)malloc(sizeof(SimpleInode));
  if (dir_inode == NULL) { printf("Cannot allocate memory for directory\n"); exit(1); }
  dir_inode->type = DIR_TYPE;
  dir_inode->path = strdup(path);
  dir_inode->size = 0;
  dir_inode->next = NULL;

  DIR * dir_stream = opendir(path);
  if (dir_stream == NULL) { printf("Cannot open directory\n"); exit(1); }

  SimpleInode * children = NULL;

  struct dirent* dir_entry = readdir(dir_stream);
  while (dir_entry != NULL) {

    // Skip if meta directory
    if (strcmp(dir_entry->d_name, ".") == 0
	|| strcmp(dir_entry->d_name, "..") == 0) {
      dir_entry = readdir(dir_stream);
      continue;
    }

    // Prepare new path name
    int curr_path_len = strlen(path);
    int entry_name_len = strlen(dir_entry->d_name);
    int concat_len = curr_path_len + 1 + entry_name_len;
    char* new_path = (char*)malloc(sizeof(char) * (concat_len + 1));
    strcpy(new_path, path);
    strcat(new_path, "/");
    strcat(new_path, dir_entry->d_name);

    // Prepare data structures for new entry
    struct stat entry_stat;
    SimpleInode* new_inode;
    int stat_result = stat(new_path, &entry_stat);
    if (stat_result != 0) { printf("Cannot access inode\n"); exit(1); }

    // Process child entry
    if ((entry_stat.st_mode & S_IFMT) == S_IFREG) {
      printf("Process a reg: %s\n", new_path);
      new_inode = parseReg(new_path);
    } else if ((entry_stat.st_mode & S_IFMT) == S_IFDIR) {
      printf("Process a dir: %s\n", new_path);
      new_inode = parseDir(new_path);
    } else {
      dir_entry = readdir(dir_stream);
      continue;
    }
    new_inode->next = children;
    children = new_inode;
    dir_inode->size += children->size;
    free(new_path);
    dir_entry = readdir(dir_stream);
  }

  dir_inode->detail.dir_detail.children = children;

  return dir_inode;
}
