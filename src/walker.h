#ifndef _WALKER
#define _WALKER

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

void printInode(SimpleInode * inode, int depth);
SimpleInode * parseReg(const char * path);
SimpleInode * parseDir(const char * path);

#endif
