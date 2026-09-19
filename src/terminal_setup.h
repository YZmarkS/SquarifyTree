#ifndef _TERMINAL_SETUP
#define _TERMINAL_SETUP

#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>
#include <termios.h>
#include <unistd.h>
#include <sys/stat.h>

void disableRawMode();

void normalTearDown();

void enableRawMode();

void terminalSetup();

#endif
