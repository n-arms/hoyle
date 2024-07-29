
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

void id(void *_result, void *x, void *t) {
  void *y = malloc(((_witness *) t) -> size);
  (((_witness *) t) -> copy)(y, x, ((_witness *) t) -> extra);
  (((_witness *) t) -> destroy)(x, ((_witness *) t) -> extra);
  (((_witness *) t) -> move)(_result, y, ((_witness *) t) -> extra);
  _destroy_type(t);
}

void poly_let(void *_result) {
  char _1[8];
  *(double *) _1 = 3;
  char _2[sizeof(_witness)];
  F64(_2);
  char _0[8];
  char _3[8];
  memmove(_3, _1, 8);
  char _4[sizeof(_witness)];
  _copy_type(_4, _2);
  id(_0, _3, _4);
  _destroy_type(_2);
  memmove(_result, _0, 8);
}
