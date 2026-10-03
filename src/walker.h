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

#include "simple_inode.h"

void printInode(SimpleInode * inode, int depth);

SimpleInode* parseReg(const char * path);

SimpleInode* parseDir(const char * path);

#endif
