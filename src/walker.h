#ifndef _WALKER
#define _WALKER

#define _LARGEFILE64_SOURCE

#include <stdlib.h>
#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <libgen.h>
#include <string.h>

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
  off64_t size;
  union {
    RegDetail reg_detail;
    DirDetail dir_detail;
  } detail;
};

#endif
