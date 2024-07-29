
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
  (((_witness *) t) -> move)(_result, x, ((_witness *) t) -> extra);
  _destroy_type(t);
}

void literal(void *_result) {
  char _0[8];
  *(double *) _0 = 3;
  memmove(_result, _0, 8);
}

void chained_poly_id_literal(void *_result) {
  char _2[8];
  literal(_2);
  char _3[sizeof(_witness)];
  F64(_3);
  char _1[8];
  char _5[8];
  memmove(_5, _2, 8);
  char _6[sizeof(_witness)];
  _copy_type(_6, _3);
  id(_1, _5, _6);
  _destroy_type(_3);
  char _4[sizeof(_witness)];
  F64(_4);
  char _0[8];
  char _7[8];
  memmove(_7, _1, 8);
  char _8[sizeof(_witness)];
  _copy_type(_8, _4);
  id(_0, _7, _8);
  _destroy_type(_4);
  memmove(_result, _0, 8);
}
