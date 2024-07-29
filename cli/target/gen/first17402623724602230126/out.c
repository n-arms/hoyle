
#include <string.h>
#include <limits.h>
#include <stdlib.h>
typedef struct _witness {
  void (*move)(void *, void *, void *);
  void (*copy)(void *, void *, void *);
  void (*destroy)(void *, void *);
  void *extra;
} _witness;

void _move_F64(void *dest, void *src, void *extra) {
  memmove(dest, src, 8);
}

void _destroy_F64(void *dest, void *extra) {}

void F64(void *_result) {
  _witness *result = _result;
  result -> move = _move_F64;
  result -> copy = _move_F64;
  result -> destroy = _destroy_F64;
  result -> extra = NULL;
}

void _move_type(void *dest, void *src) {
    memmove(dest, src, 32);
}

void _copy_type(void *dest, void *src) {
    _witness *typ = src;
    if (typ -> extra != NULL) {
        unsigned long long *counter = typ -> extra;
        if (*counter == ULONG_MAX) {
            exit(1);
        } else {
            *counter += 1;
        }
    }
    memmove(dest, src, 32);
}

void _destroy_type(void *src) {
    _witness *typ = src;
    if (typ -> extra != NULL) {
        unsigned long long *counter = typ -> extra;
        if (*counter == 0) {
            free(typ -> extra);
        } else {
            *counter -= 1;
        }
    }
}

void first_of(void *_result, void *x, void *y, void *a, void *b) {
  (((_witness *) b) -> destroy)(y, ((_witness *) b) -> extra);
  _destroy_type(b);
  (((_witness *) a) -> move)(_result, x, ((_witness *) a) -> extra);
  _destroy_type(a);
}

void first(void *_result) {
  char _1[8];
  *(double *) _1 = 3;
  char _2[8];
  *(double *) _2 = 4;
  char _3[sizeof(_witness)];
  F64(_3);
  char _4[sizeof(_witness)];
  F64(_4);
  char _0[8];
  char _5[8];
  memmove(_5, _1, 8);
  char _6[8];
  memmove(_6, _2, 8);
  char _7[sizeof(_witness)];
  _copy_type(_7, _3);
  char _8[sizeof(_witness)];
  _copy_type(_8, _4);
  first_of(_0, _5, _6, _7, _8);
  _destroy_type(_4);
  _destroy_type(_3);
  memmove(_result, _0, 8);
}
