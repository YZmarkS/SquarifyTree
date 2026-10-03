#ifndef _SIMPLE_INODE
#define _SIMPLE_INODE

#ifndef _LARGEFILE64_SOURCE
#define _LARGEFILE64_SOURCE
#endif

#include <stdlib.h>
#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <libgen.h>
#include <string.h>
#include <dirent.h>

typedef enum PathTypeEnum {
  REG_TYPE,
  DIR_TYPE
} PathType;

typedef struct RegDetail RegDetail;
typedef struct DirDetail DirDetail;

typedef struct SimpleInode SimpleInode;
typedef struct InodePtrList InodePtrList;

struct RegDetail {
  off_t size;
};

struct DirDetail {
  SimpleInode * children;
};

struct SimpleInode {
  PathType type;
  char * path;
  off_t size;
  union {
    RegDetail reg_detail;
    DirDetail dir_detail;
  } detail;
  SimpleInode * next;
};

typedef struct InodePtrList {
  SimpleInode* inode;
  InodePtrList* next;
} InodePtrList;

// One must maintain the invariant that len(content) = length
typedef struct InodePtrSlice {
  SimpleInode** content;
  long length;
} InodePtrSlice;

void printInode(SimpleInode inode[], int depth);

SimpleInode* sortList(SimpleInode* list);

long inodeListLength(SimpleInode* list);

off_t sumInodeListSize(SimpleInode* list);

off_t sumFirstNInodesSize(SimpleInode* list, long n);

#endif
